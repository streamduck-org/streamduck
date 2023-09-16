using System;
using System.Threading;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.ApplicationLifetimes;
using Avalonia.ReactiveUI;
using NLog;
using NLog.Config;
using NLog.Layouts;
using NLog.Targets;
using Streamduck.Configuration;
using Streamduck.UI;
using Streamduck.UI.ViewModels;

// Setting up logger

namespace Streamduck; 

internal class Program {
	// Avalonia configuration, don't remove; also used by visual designer.
	public static AppBuilder BuildAvaloniaApp()
		=> AppBuilder.Configure<UIApp>()
			.UsePlatformDetect()
			.WithInterFont()
			.LogToTrace()
			.UseReactiveUI();
	
	public static async Task Main(string[] args) {
		var cts = new CancellationTokenSource();
		
		var logConfig = new LoggingConfiguration();

		logConfig.AddRule(LogLevel.Debug, LogLevel.Fatal, new ColoredConsoleTarget {
			Layout = Layout.FromString("${longdate} ${level:uppercase=true} (${logger}): ${message}${onexception:inner=\\: ${exception}}")
		});
		System.Diagnostics.Trace.Listeners.Add(new NLogTraceListener { Name = "SysTrace"});

		LogManager.Configuration = logConfig;

		var L = LogManager.GetCurrentClassLogger();

		// Initializing Streamduck
		var streamduck = new App();
		await streamduck.Init();

		// Starting Streamduck
		_ = Task.Run(async () => {
			try {
				await streamduck.Run(cts);
			} catch (TaskCanceledException) {
				// Ignored
			} catch (Exception e) {
				L!.Error(e, "Critical Error!");
			} finally {
				var config = await Config.Get();
				await config.SaveConfig();
			}
		}, cts.Token);
		
		// Starting UI
		BuildAvaloniaApp()
			.Start((app, strings) => {
				L.Info("Starting UI...");
				if (app is not UIApp uiApp) return;
				
				L.Debug("Setting Streamduck reference");
				uiApp.StreamduckApp = streamduck;

				L.Debug("Creating main window");
				uiApp.MainWindow = new MainWindow();
				uiApp.MainWindow.DataContext = new MainWindowViewModel();

				uiApp.CancellationTokenSource = cts;
				
				L.Debug("Running UI Loop");
				uiApp.Run(cts.Token);
				L.Debug("UI Loop ended");
			}, Array.Empty<string>());

		Environment.Exit(0);
	}
}