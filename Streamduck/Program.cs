using NLog;
using NLog.Config;
using NLog.Layouts;
using NLog.Targets;
using Streamduck;

// Setting up logger
var logConfig = new LoggingConfiguration();

logConfig.AddRule(LogLevel.Debug, LogLevel.Fatal, new ColoredConsoleTarget {
	Layout = Layout.FromString("${longdate} ${level:uppercase=true} (${logger}): ${message}")
});

LogManager.Configuration = logConfig;

var L = LogManager.GetCurrentClassLogger();

// Initializing Streamduck
var streamduck = new App();
await streamduck.Init();

// Starting Streamduck
await streamduck.Run();