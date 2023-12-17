using System;
using System.Diagnostics;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using Avalonia.ReactiveUI;
using NLog;
using NLog.Config;
using NLog.Layouts;
using NLog.Targets;
using Streamduck.Configuration;
using Streamduck.Devices;
using Streamduck.Fields;
using Streamduck.Plugins;
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
	
	private class TestOptions {
		public string Label => "test";
	}

	public static async Task Main(string[] args) {
		var fields = FieldReflector.AnalyzeObject(new TestOptions()).ToArray();
		Console.WriteLine("Analyzed object");

		foreach (var field in fields) {
			Console.WriteLine(field.Title);
		}
		
		var cts = new CancellationTokenSource();

		var logConfig = new LoggingConfiguration();

		logConfig.AddRule(LogLevel.Debug, LogLevel.Fatal, new ColoredConsoleTarget {
			Layout = Layout.FromString(
				"${longdate} ${level:uppercase=true} (${logger}): ${message}${onexception:inner=\\: ${exception}}")
		});
		Trace.Listeners.Add(new NLogTraceListener { Name = "SysTrace" });

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