// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System.Text.Json;
using Streamduck.Plugins;

namespace Streamduck.Socket;

public class SocketMessage {
	public NamespacedName Name { get; set; }
	public JsonElement? Data { get; set; }
	public string? RequestID { get; set; }
}