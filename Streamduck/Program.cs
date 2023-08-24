using System.Linq;
using ElgatoStreamDeck;
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

// Temporary testing of ElgatoStreamDeck
var deviceManager = DeviceManager.Get();

foreach (var pair in deviceManager.ListDevices()) L.Info("Found {0}", pair);

var (firstDeviceKind, firstDeviceSerial) = deviceManager.ListDevices().First();

var device = deviceManager.ConnectDevice(firstDeviceKind, firstDeviceSerial);
device.SetBrightness(35);

// Initializing Streamduck
var streamduck = new App();
streamduck.Init();

// Starting Streamduck
await streamduck.Run();