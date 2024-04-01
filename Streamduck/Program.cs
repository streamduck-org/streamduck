// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System;
using System.Diagnostics;
using System.Net;
using System.Threading;
using System.Threading.Tasks;
using NLog;
using NLog.Config;
using NLog.Layouts;
using NLog.Targets;
using Streamduck.Configuration;
using Streamduck.Socket;

// Setting up logger

namespace Streamduck;

internal class Program {
	public static async Task Main(string[] args) {
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

		// Starting API
		var config = await Config.Get();

		var server = new Server(
			config.OpenToInternet
				? IPAddress.Any
				: IPAddress.Loopback, config.WebSocketPort) {
			AppInstance = streamduck
		};

		server.Start();

		await Task.Delay(-1, cts.Token);
	}
}