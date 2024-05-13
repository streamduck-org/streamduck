// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System;
using System.Text.Json;
using NetCoreServer;
using Streamduck.Plugins;

namespace Streamduck.Socket;

public class SessionRequester(WsSession Session, SocketMessage Message) : SocketRequester {
	public override SocketMessage Message { get; } = Message;

	public override void SendBack(object? data) {
		var response = JsonSerializer.Serialize(
			new Response {
				Data = data,
				Name = Message.Name,
				RequestID = Message.RequestID
			}
		);
		Session.SendTextAsync(response);
	}

	public override void SendBackError(string message) {
		SendBack(new SocketError(message));
	}

	public override T? ParseData<T>() where T : class {
		if (Message.Data is not { } data) {
			SendBack(new SocketError("Data was missing for request that needs data"));
			return null;
		}

		try {
			return data.Deserialize<T>();
		} catch (JsonException e) {
			SendBack(new SocketError(e.Message));
			return null;
		}
	}
}

internal class Response {
	public string? RequestID { get; set; }
	public required NamespacedName Name { get; set; }
	public required object? Data { get; set; }
}