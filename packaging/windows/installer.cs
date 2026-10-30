using System;
using System.Diagnostics;
using System.Drawing;
using System.IO;
using System.Reflection;
using System.Runtime.InteropServices;
using System.Threading;
using System.Windows.Forms;
using Microsoft.Win32;

namespace BorizonInstaller
{
    static class Program
    {
        [STAThread]
        static int Main(string[] args)
        {
            Application.EnableVisualStyles();
            Application.SetCompatibleTextRenderingDefault(false);

            bool isUninstall = false;
            bool isSilent = false;

            string exeName = Process.GetCurrentProcess().ProcessName.ToLowerInvariant();
            if (exeName.Contains("uninstall")) isUninstall = true;

            foreach (string arg in args)
            {
                string a = arg.ToLowerInvariant().Trim();
                if (a == "/s" || a == "-s" || a == "/silent" || a == "--silent") isSilent = true;
                if (a == "/uninstall" || a == "-uninstall") isUninstall = true;
            }

            if (isUninstall)
            {
                if (isSilent)
                {
                    PerformUninstall(null);
                    return 0;
                }
                Application.Run(new UninstallForm());
                return 0;
            }
            else
            {
                if (isSilent)
                {
                    PerformInstall(null);
                    return 0;
                }
                Application.Run(new InstallerForm());
                return 0;
            }
        }

        public static string InstallDir
        {
            get
            {
                string local = Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData);
                return Path.Combine(local, "Programs", "Borizon");
            }
        }

        public static void CreateShortcut(string shortcutPath, string targetPath, string workingDir, string description)
        {
            try
            {
                Type shellType = Type.GetTypeFromProgID("WScript.Shell");
                if (shellType == null) return;
                dynamic shell = Activator.CreateInstance(shellType);
                dynamic shortcut = shell.CreateShortcut(shortcutPath);
                shortcut.TargetPath = targetPath;
                shortcut.WorkingDirectory = workingDir;
                shortcut.Description = description;
                shortcut.IconLocation = targetPath + ",0";
                shortcut.Save();
            }
            catch { }
        }

        public static void PerformInstall(Action<string, int> progress)
        {
            string dir = InstallDir;
            if (progress != null) progress("Checking for running instances...", 10);
            KillProcess("horizon-launcher");

            if (!Directory.Exists(dir))
            {
                Directory.CreateDirectory(dir);
            }

            if (progress != null) progress("Extracting: horizon-launcher.exe... 100%", 40);
            string destExe = Path.Combine(dir, "horizon-launcher.exe");

            // Extract payload instantly
            var asm = Assembly.GetExecutingAssembly();
            using (Stream s = asm.GetManifestResourceStream("HorizonLauncher"))
            {
                if (s != null)
                {
                    using (FileStream fs = new FileStream(destExe, FileMode.Create, FileAccess.Write, FileShare.None, 4 * 1024 * 1024))
                    {
                        s.CopyTo(fs);
                    }
                }
            }

            // Also copy current installer as uninstaller
            string currentExe = Process.GetCurrentProcess().MainModule.FileName;
            string uninstExe = Path.Combine(dir, "uninstall.exe");
            try
            {
                File.Copy(currentExe, uninstExe, true);
            }
            catch { }

            if (progress != null) progress("Creating shortcuts...", 70);
            string desktop = Environment.GetFolderPath(Environment.SpecialFolder.DesktopDirectory);
            string startMenu = Environment.GetFolderPath(Environment.SpecialFolder.StartMenu);
            string programs = Path.Combine(startMenu, "Programs", "Borizon");

            if (!Directory.Exists(programs))
            {
                Directory.CreateDirectory(programs);
            }

            CreateShortcut(Path.Combine(desktop, "Borizon.lnk"), destExe, dir, "Borizon");
            CreateShortcut(Path.Combine(programs, "Borizon.lnk"), destExe, dir, "Borizon");
            CreateShortcut(Path.Combine(programs, "Uninstall Borizon.lnk"), uninstExe, dir, "Uninstall Borizon");

            // Also update legacy shortcut if present so old click works seamlessly
            string legacyDesktop = Path.Combine(desktop, "Horizon Offline.lnk");
            if (File.Exists(legacyDesktop))
            {
                CreateShortcut(legacyDesktop, destExe, dir, "Borizon");
            }

            if (progress != null) progress("Registering application...", 90);
            RegisterUninstall(dir, destExe, uninstExe);

            if (progress != null) progress("Completed", 100);
        }

        public static void PerformUninstall(Action<string, int> progress)
        {
            if (progress != null) progress("Closing running instances...", 20);
            KillProcess("horizon-launcher");

            string dir = InstallDir;
            string desktop = Environment.GetFolderPath(Environment.SpecialFolder.DesktopDirectory);
            string startMenu = Environment.GetFolderPath(Environment.SpecialFolder.StartMenu);
            string programs = Path.Combine(startMenu, "Programs", "Borizon");

            if (progress != null) progress("Deleting shortcuts...", 50);
            try { File.Delete(Path.Combine(desktop, "Borizon.lnk")); } catch { }
            try { File.Delete(Path.Combine(desktop, "Horizon Offline.lnk")); } catch { }
            try { File.Delete(Path.Combine(programs, "Borizon.lnk")); } catch { }
            try { File.Delete(Path.Combine(programs, "Uninstall Borizon.lnk")); } catch { }
            try { Directory.Delete(programs, true); } catch { }

            if (progress != null) progress("Removing registry keys...", 80);
            UnregisterUninstall();

            if (progress != null) progress("Removing files...", 95);
            try
            {
                string exe = Path.Combine(dir, "horizon-launcher.exe");
                if (File.Exists(exe)) File.Delete(exe);
            }
            catch { }

            // Schedule self delete via cmd
            string currentExe = Process.GetCurrentProcess().MainModule.FileName;
            ProcessStartInfo psi = new ProcessStartInfo("cmd.exe", "/C choice /C Y /N /D Y /T 1 & rmdir /S /Q \"" + dir + "\"")
            {
                WindowStyle = ProcessWindowStyle.Hidden,
                CreateNoWindow = true
            };
            try { Process.Start(psi); } catch { }

            if (progress != null) progress("Uninstalled successfully", 100);
        }

        private static void KillProcess(string name)
        {
            try
            {
                foreach (var p in Process.GetProcessesByName(name))
                {
                    try { p.Kill(); p.WaitForExit(1000); } catch { }
                }
            }
            catch { }
        }

        private static void RegisterUninstall(string dir, string exe, string uninst)
        {
            try
            {
                using (var k = Registry.CurrentUser.CreateSubKey(@"Software\Microsoft\Windows\CurrentVersion\Uninstall\Borizon"))
                {
                    if (k != null)
                    {
                        k.SetValue("DisplayName", "Borizon");
                        k.SetValue("DisplayVersion", "0.2.0");
                        k.SetValue("Publisher", "Borizon Contributors");
                        k.SetValue("URLInfoAbout", "https://jabirking-ux.github.io/horizon-offline-site/");
                        k.SetValue("DisplayIcon", exe + ",0");
                        k.SetValue("UninstallString", "\"" + uninst + "\"");
                        k.SetValue("QuietUninstallString", "\"" + uninst + "\" /S");
                        k.SetValue("InstallLocation", dir);
                        k.SetValue("NoModify", 1, RegistryValueKind.DWord);
                        k.SetValue("NoRepair", 1, RegistryValueKind.DWord);
                    }
                }
                using (var app = Registry.CurrentUser.CreateSubKey(@"Software\Microsoft\Windows\CurrentVersion\App Paths\horizon-launcher.exe"))
                {
                    if (app != null)
                    {
                        app.SetValue("", exe);
                        app.SetValue("Path", dir);
                    }
                }
            }
            catch { }
        }

        private static void UnregisterUninstall()
        {
            try
            {
                Registry.CurrentUser.DeleteSubKeyTree(@"Software\Microsoft\Windows\CurrentVersion\Uninstall\Borizon", false);
                Registry.CurrentUser.DeleteSubKeyTree(@"Software\Microsoft\Windows\CurrentVersion\App Paths\horizon-launcher.exe", false);
            }
            catch { }
        }
    }

    public class InstallerForm : Form
    {
        private Panel headerPanel;
        private Label titleLabel;
        private Label subtitleLabel;
        private PictureBox iconBox;
        private Label statusLabel;
        private ProgressBar progressBar;
        private ListBox detailsBox;
        private Button detailsButton;
        private CheckBox runCheckbox;
        private Button actionButton;
        private Button cancelButton;
        private Panel bottomDivider;
        private Panel topDivider;
        private System.Windows.Forms.Timer timer;
        private int step = 0;

        public InstallerForm()
        {
            InitializeComponent();
        }

        private void InitializeComponent()
        {
            this.Text = "Borizon 0.2.0 Setup";
            this.ClientSize = new Size(500, 385);
            this.FormBorderStyle = FormBorderStyle.FixedDialog;
            this.MaximizeBox = false;
            this.MinimizeBox = true;
            this.StartPosition = FormStartPosition.CenterScreen;
            this.Font = new Font("Segoe UI", 9F, FontStyle.Regular);

            try
            {
                this.Icon = Icon.ExtractAssociatedIcon(Process.GetCurrentProcess().MainModule.FileName);
            }
            catch { }

            // Header panel (classic NSIS white banner)
            headerPanel = new Panel
            {
                Dock = DockStyle.Top,
                Height = 65,
                BackColor = Color.White
            };

            titleLabel = new Label
            {
                Text = "Installing",
                Font = new Font("Segoe UI", 9F, FontStyle.Bold),
                Location = new Point(20, 15),
                AutoSize = true
            };

            subtitleLabel = new Label
            {
                Text = "Please wait while Borizon is being installed.",
                Font = new Font("Segoe UI", 8.25F, FontStyle.Regular),
                Location = new Point(35, 34),
                AutoSize = true,
                ForeColor = Color.FromArgb(70, 70, 70)
            };

            iconBox = new PictureBox
            {
                Size = new Size(32, 32),
                Location = new Point(445, 16),
                SizeMode = PictureBoxSizeMode.StretchImage
            };
            if (this.Icon != null) iconBox.Image = this.Icon.ToBitmap();

            topDivider = new Panel
            {
                Dock = DockStyle.Bottom,
                Height = 1,
                BackColor = Color.FromArgb(215, 215, 215)
            };

            headerPanel.Controls.Add(titleLabel);
            headerPanel.Controls.Add(subtitleLabel);
            headerPanel.Controls.Add(iconBox);
            headerPanel.Controls.Add(topDivider);

            // Status label
            statusLabel = new Label
            {
                Text = "Extracting: horizon-launcher.exe...",
                Location = new Point(20, 80),
                Size = new Size(460, 20)
            };

            // Progress bar
            progressBar = new ProgressBar
            {
                Location = new Point(20, 102),
                Size = new Size(460, 20),
                Minimum = 0,
                Maximum = 100,
                Value = 0
            };

            // Details ListBox
            detailsBox = new ListBox
            {
                Location = new Point(20, 130),
                Size = new Size(460, 165),
                Font = new Font("Consolas", 8.25F, FontStyle.Regular),
                IntegralHeight = false
            };

            // Bottom Divider
            bottomDivider = new Panel
            {
                Location = new Point(0, 335),
                Size = new Size(500, 1),
                BackColor = Color.FromArgb(215, 215, 215)
            };

            // "Show details" toggle button
            detailsButton = new Button
            {
                Text = "Show details",
                Location = new Point(20, 345),
                Size = new Size(85, 26)
            };
            detailsButton.Click += (s, e) =>
            {
                detailsBox.Visible = !detailsBox.Visible;
                detailsButton.Text = detailsBox.Visible ? "Hide details" : "Show details";
            };

            // Run Checkbox
            runCheckbox = new CheckBox
            {
                Text = "Run Borizon",
                Location = new Point(120, 348),
                AutoSize = true,
                Checked = true
            };

            // Cancel / Finish button
            actionButton = new Button
            {
                Text = "Cancel",
                Location = new Point(400, 345),
                Size = new Size(80, 26)
            };
            actionButton.Click += (s, e) =>
            {
                if (actionButton.Text == "Close" || actionButton.Text == "Finish")
                {
                    if (runCheckbox.Checked)
                    {
                        try
                        {
                            string exe = Path.Combine(Program.InstallDir, "horizon-launcher.exe");
                            if (File.Exists(exe)) Process.Start(exe);
                        }
                        catch { }
                    }
                    this.Close();
                }
                else
                {
                    this.Close();
                }
            };

            this.Controls.Add(headerPanel);
            this.Controls.Add(statusLabel);
            this.Controls.Add(progressBar);
            this.Controls.Add(detailsBox);
            this.Controls.Add(bottomDivider);
            this.Controls.Add(detailsButton);
            this.Controls.Add(runCheckbox);
            this.Controls.Add(actionButton);

            // Timer to animate instant extraction steps smoothly across ~250ms
            timer = new System.Windows.Forms.Timer { Interval = 40 };
            timer.Tick += Timer_Tick;
            this.Shown += (s, e) => timer.Start();
        }

        private void Timer_Tick(object sender, EventArgs e)
        {
            step++;
            if (step == 1)
            {
                detailsBox.Items.Add("Output folder: " + Program.InstallDir);
                statusLabel.Text = "Extracting files to " + Program.InstallDir + "...";
                progressBar.Value = 25;
            }
            else if (step == 2)
            {
                // Run the actual instant payload install
                Program.PerformInstall((msg, pct) =>
                {
                    detailsBox.Items.Add(msg);
                });
                progressBar.Value = 75;
                statusLabel.Text = "Creating shortcuts...";
            }
            else if (step == 3)
            {
                progressBar.Value = 100;
                statusLabel.Text = "Completed";
                titleLabel.Text = "Installation Complete";
                subtitleLabel.Text = "Setup was completed successfully.";
                actionButton.Text = "Finish";
                actionButton.Focus();
                timer.Stop();
            }
        }
    }

    public class UninstallForm : Form
    {
        private Panel headerPanel;
        private Label titleLabel;
        private Label subtitleLabel;
        private PictureBox iconBox;
        private Label statusLabel;
        private ProgressBar progressBar;
        private Button closeButton;
        private Panel bottomDivider;
        private Panel topDivider;
        private System.Windows.Forms.Timer timer;
        private int step = 0;

        public UninstallForm()
        {
            this.Text = "Borizon Uninstall";
            this.ClientSize = new Size(500, 300);
            this.FormBorderStyle = FormBorderStyle.FixedDialog;
            this.MaximizeBox = false;
            this.MinimizeBox = true;
            this.StartPosition = FormStartPosition.CenterScreen;
            this.Font = new Font("Segoe UI", 9F, FontStyle.Regular);

            headerPanel = new Panel { Dock = DockStyle.Top, Height = 65, BackColor = Color.White };
            titleLabel = new Label { Text = "Uninstall Borizon", Font = new Font("Segoe UI", 9F, FontStyle.Bold), Location = new Point(20, 15), AutoSize = true };
            subtitleLabel = new Label { Text = "Remove Borizon from your computer.", Font = new Font("Segoe UI", 8.25F, FontStyle.Regular), Location = new Point(35, 34), AutoSize = true, ForeColor = Color.FromArgb(70, 70, 70) };
            iconBox = new PictureBox { Size = new Size(32, 32), Location = new Point(445, 16), SizeMode = PictureBoxSizeMode.StretchImage };
            topDivider = new Panel { Dock = DockStyle.Bottom, Height = 1, BackColor = Color.FromArgb(215, 215, 215) };
            headerPanel.Controls.AddRange(new Control[] { titleLabel, subtitleLabel, iconBox, topDivider });

            statusLabel = new Label { Text = "Uninstalling Borizon...", Location = new Point(20, 100), Size = new Size(460, 20) };
            progressBar = new ProgressBar { Location = new Point(20, 125), Size = new Size(460, 20), Maximum = 100 };

            bottomDivider = new Panel { Location = new Point(0, 250), Size = new Size(500, 1), BackColor = Color.FromArgb(215, 215, 215) };
            closeButton = new Button { Text = "Close", Location = new Point(400, 260), Size = new Size(80, 26), Enabled = false };
            closeButton.Click += (s, e) => this.Close();

            this.Controls.AddRange(new Control[] { headerPanel, statusLabel, progressBar, bottomDivider, closeButton });

            timer = new System.Windows.Forms.Timer { Interval = 60 };
            timer.Tick += (s, e) =>
            {
                step++;
                if (step == 1) { progressBar.Value = 30; }
                else if (step == 2)
                {
                    Program.PerformUninstall(null);
                    progressBar.Value = 100;
                    statusLabel.Text = "Uninstallation Complete.";
                    titleLabel.Text = "Uninstalled";
                    subtitleLabel.Text = "Borizon has been removed from your computer.";
                    closeButton.Enabled = true;
                    closeButton.Focus();
                    timer.Stop();
                }
            };
            this.Shown += (s, e) => timer.Start();
        }
    }
}
