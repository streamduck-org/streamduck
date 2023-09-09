using System;
using System.Text.Json;
using NLog;
using NLog.Config;
using NLog.Layouts;
using NLog.Targets;
using Streamduck;
using Streamduck.Configuration;
using Streamduck.Definitions.UI;

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
try {
	await streamduck.Run();
} catch (Exception e) {
	L.Error("Critical Error! \n{}", e);
} finally {
	var config = await Config.Get();
	await config.SaveConfig();
}