using System;
using System.Collections.Generic;
using System.Threading.Tasks;
using Streamduck.Api;
using Streamduck.Scripting;

namespace Streamduck.Plugins;

public abstract class Plugin : INamed {
	public abstract string Name { get; }

	public virtual IEnumerable<Driver> Drivers { get; } = Array.Empty<Driver>();
	public virtual IEnumerable<PluginAction> Actions { get; } = Array.Empty<PluginAction>();
	public virtual IEnumerable<PluginFunction> Functions { get; } = Array.Empty<PluginFunction>();
	public virtual IEnumerable<AsyncPluginAction> AsyncActions { get; } = Array.Empty<AsyncPluginAction>();
	public virtual IEnumerable<AsyncPluginFunction> AsyncFunctions { get; } = Array.Empty<AsyncPluginFunction>();
	public virtual IEnumerable<ScriptingSystem> ScriptingSystems { get; } = Array.Empty<ScriptingSystem>();

	public virtual Task OnPluginsLoaded(IPluginQuery pluginQuery) => Task.CompletedTask;

	public virtual Task OnNewPluginsLoaded(IEnumerable<Plugin> newPlugins, IPluginQuery pluginQuery) =>
		Task.CompletedTask;
}