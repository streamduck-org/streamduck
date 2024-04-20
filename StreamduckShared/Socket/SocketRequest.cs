// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System.Threading.Tasks;
using Streamduck.Interfaces;

namespace Streamduck.Socket;

/**
 * Socket requests that can be defined by plugins for custom socket behaviors
 */
public abstract class SocketRequest : INamed {
	public abstract string Name { get; }

	public abstract Task Received(SocketRequester request);
}

/**
 * Socket requests that can be defined by plugins for custom socket behaviors, with automatic data parsing
 */
public abstract class SocketRequest<T> : SocketRequest where T : class {
	public override Task Received(SocketRequester request) {
		if (request.ParseData<T>() is { } data) return Received(request, data);
		return Task.CompletedTask;
	}

	public abstract Task Received(SocketRequester request, T data);
}