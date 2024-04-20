// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

namespace Streamduck.Socket;

/**
 * Represents the client that sent the request, contains message and helper methods to send back or serialize data
 */
public abstract class SocketRequester {
	public abstract SocketMessage Message { get; }
	public abstract void SendBack(object? data);
	public abstract void SendBackError(string message);
	public abstract T? ParseData<T>() where T : class;
}