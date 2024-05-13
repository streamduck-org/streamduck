// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System;
using System.Collections.Generic;
using System.Net;
using System.Text;
using System.Text.Json;
using System.Text.Json.Serialization;
using System.Threading.Tasks;
using NetCoreServer;
using NLog;
using Streamduck.Plugins;

namespace Streamduck.Socket;

public class Session : WsSession {
	private static readonly Logger _l = LogManager.GetCurrentClassLogger();

	private readonly Server _server;
	// private readonly StringBuilder _requestBuffer = new();

	private readonly Dictionary<Plugin, Action<string, object>> _listeners = [];

	public Session(Server server) : base(server) {
		_server = server;
		_l.Info($"Client {Id} connection started");

		foreach (var plugin in _server.AppInstance.Plugins.AllPlugins()) {
			Action<string, object> action = (name, data) => SendEventAsync(plugin, name, data);
			_listeners[plugin] = action;
			plugin.EventEmitted += action;
		}
	}

	public override void OnWsConnected(HttpResponse response) {
		_l.Info($"Client {Id} connected");
	}

	public override void OnWsDisconnected() {
		_l.Info($"Client {Id} disconnected");
		foreach (var (plugin, action) in _listeners) plugin.EventEmitted -= action;
	}

	public override void OnWsReceived(byte[] buffer, long offset, long size) {
		var message = Encoding.UTF8.GetString(buffer, (int)offset, (int)size);
		ReceivePacket(message);

		// while (true) {
		// 	var nullIndex = message.IndexOf('\u0004');
		//
		// 	if (nullIndex >= 0) {
		// 		var left = message[..nullIndex];
		// 		ReceivePacket(_requestBuffer + left);
		// 		_requestBuffer.Clear();
		// 		message = message[(nullIndex + 1)..];
		// 	} else {
		// 		_requestBuffer.Append(message);
		// 		break;
		// 	}
		// }
	}

	public void ReceivePacket(string packet) {
		Console.WriteLine(packet);
		try {
			var message = JsonSerializer.Deserialize<SocketMessage>(packet);

			if (message is null) {
				SendErrorAsync(new SocketError("Failed to parse"));
				return;
			}

			if (_server.AppInstance.PluginCollection!.SpecificSocketRequest(message.Name) is not { } request) {
				SendErrorAsync(new SocketError($"Request with name '{message.Name}' not found"));
				return;
			}

			Task.Run(
				async () => {
					await request.Instance.Received(new SessionRequester(this, message)).ConfigureAwait(false);
				}
			);
		} catch (JsonException e) {
			SendErrorAsync(new SocketError(e.ToString()));
		}
	}

	internal void SendErrorAsync(SocketError error) {
		SendTextAsync(JsonSerializer.Serialize(error));
	}

	internal void SendEventAsync(Plugin plugin, string name, object data) {
		SendTextAsync(
			JsonSerializer.Serialize(
				new Event {
					PluginName = plugin.Name,
					EventName = name,
					Data = data
				}
			)
		);
	}
}

public class Event {
	public string PluginName { get; set; } = "unknown";
	public string EventName { get; set; } = "unknown";
	public object? Data { get; set; }
}

internal readonly struct SocketError(string Error) {
	[JsonInclude] public string Error { get; } = Error;

	public override int GetHashCode() {
		unchecked {
			var hash = 17;
			hash = hash * 23 + Error.GetHashCode();
			return hash;
		}
	}
}

public class Server : WsServer {
	private static readonly Logger _l = LogManager.GetCurrentClassLogger();

	public Server(IPAddress address, int port) : base(address, port) { }
	public Server(string address, int port) : base(address, port) { }
	public Server(DnsEndPoint endpoint) : base(endpoint) { }
	public Server(IPEndPoint endpoint) : base(endpoint) { }

	public required App AppInstance { get; init; }

	protected override void OnStarted() {
		_l.Info($"Listening for websocket connections at {Address}:{Port}");
	}

	protected override TcpSession CreateSession() => new Session(this);

	protected override void OnError(System.Net.Sockets.SocketError error) {
		_l.Error($"WebSocket error: {error}");
	}
}