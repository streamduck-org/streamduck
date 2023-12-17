using System;
using System.Collections.Generic;
using System.Threading.Tasks;
using Streamduck.Cores;
using Streamduck.Devices;
using Streamduck.Interfaces;
using Streamduck.Rendering;

namespace Streamduck.Plugins;

public abstract class Plugin : INamed {
	public abstract string Name { get; }

	public virtual IEnumerable<Driver> Drivers { get; } = Array.Empty<Driver>();
	public virtual IEnumerable<PluginAction> Actions { get; } = Array.Empty<PluginAction>();
	public virtual IEnumerable<Renderer> Renderers { get; } = Array.Empty<Renderer>();

	public virtual Task OnPluginsLoaded(IPluginQuery pluginQuery) => Task.CompletedTask;

	public virtual Task OnNewPluginsLoaded(IEnumerable<Plugin> newPlugins, IPluginQuery pluginQuery) =>
		Task.CompletedTask;

	public virtual Task OnDeviceConnected(NamespacedDeviceIdentifier identifier, Core deviceCore) =>
		Task.CompletedTask;
	
	public virtual Task OnDeviceDisconnected(NamespacedDeviceIdentifier identifier) =>
		Task.CompletedTask;
}