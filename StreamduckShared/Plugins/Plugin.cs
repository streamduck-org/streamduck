// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System;
using System.Collections.Generic;
using System.Threading.Tasks;
using Streamduck.Actions;
using Streamduck.Cores;
using Streamduck.Devices;
using Streamduck.Interfaces;
using Streamduck.Rendering;
using Streamduck.Socket;
using Streamduck.Triggers;

namespace Streamduck.Plugins;

public abstract class Plugin : INamed {
	public virtual IEnumerable<Driver> Drivers { get; } = Array.Empty<Driver>();
	public virtual IEnumerable<PluginAction> Actions { get; } = Array.Empty<PluginAction>();
	public virtual IEnumerable<Renderer> Renderers { get; } = Array.Empty<Renderer>();
	public virtual IEnumerable<Trigger> Triggers { get; } = Array.Empty<Trigger>();
	public virtual IEnumerable<SocketRequest> SocketRequests { get; } = Array.Empty<SocketRequest>();
	public abstract string Name { get; }

	public virtual Task OnPluginsLoaded(IStreamduck streamduck) => Task.CompletedTask;

	public virtual Task OnNewPluginsLoaded(IEnumerable<Plugin> newPlugins, IStreamduck streamduck) =>
		Task.CompletedTask;

	public virtual Task OnDeviceConnected(NamespacedDeviceIdentifier identifier, Core deviceCore) =>
		Task.CompletedTask;

	public virtual Task OnDeviceDisconnected(NamespacedDeviceIdentifier identifier) =>
		Task.CompletedTask;

	public virtual Task OnDeviceAppeared(NamespacedDeviceIdentifier identifier) =>
		Task.CompletedTask;

	public virtual Task OnDeviceDisappeared(NamespacedDeviceIdentifier identifier) =>
		Task.CompletedTask;
}