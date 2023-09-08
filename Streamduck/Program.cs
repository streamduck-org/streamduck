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


var field = new Field {
	Title = "aaaaab",
	Description = "asdfasdgasd",
	Type = new FieldType.StringInput {
		Disabled = true
	},
	ValuePath = new[] { "a", "b" }
};

Console.WriteLine(JsonSerializer.Serialize(field));

// var json = """{"ValuePath":["a","b"],"Title":"aaaaab","Description":"asdfasdgasd","Type":{"$type":"Streamduck.Definitions.UI.FieldType+StringInput, StreamduckShared, Version=1.0.0.0, Culture=neutral, PublicKeyToken=null","Disabled":true}}""";
//
// var jsonData = MessagePackSerializer.ConvertFromJson(json);
// var obj = MessagePackSerializer.Deserialize<Field>(jsonData);

// Console.WriteLine(obj.Type.Type);

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