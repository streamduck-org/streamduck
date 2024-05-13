// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System;
using System.Collections.Concurrent;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using Streamduck.Actions;
using Streamduck.Configuration;
using Streamduck.Data;
using Streamduck.Plugins.Extensions;
using Streamduck.Plugins.Loaders;
using Streamduck.Rendering;
using Streamduck.Socket;
using Streamduck.Triggers;
using Streamduck.Utils;

namespace Streamduck.Plugins;

public class PluginCollection : IPluginQuery {
	private readonly ConcurrentDictionary<NamespacedName, WeakReference<Namespaced<PluginAction>>> _actionMap;
	private readonly Config _config;
	private readonly ConcurrentDictionary<NamespacedName, WeakReference<Namespaced<Driver>>> _driverMap;

	private readonly ConcurrentDictionary<string, WeakReference<WrappedPlugin>> _pluginMap;
	private readonly ConcurrentDictionary<NamespacedName, WeakReference<Namespaced<Renderer>>> _rendererMap;
	private readonly ConcurrentDictionary<NamespacedName, WeakReference<Namespaced<SocketRequest>>> _socketRequestMap;
	private readonly ConcurrentDictionary<NamespacedName, WeakReference<Namespaced<Trigger>>> _triggerMap;
	public readonly List<PluginAssembly> Plugins;

	public PluginCollection(IEnumerable<PluginAssembly> plugins, Config config) {
		_config = config;
		Plugins = plugins.ToList();
		_pluginMap = new ConcurrentDictionary<string, WeakReference<WrappedPlugin>>(
			Plugins
				.SelectMany(a => a.Plugins)
				.ToDictionary(p => p.Name, p => new WeakReference<WrappedPlugin>(p))
		);
		_driverMap = BuildMap(p => p.Drivers);
		_actionMap = BuildMap(p => p.Actions);
		_rendererMap = BuildMap(p => p.Renderers);
		_triggerMap = BuildMap(p => p.Triggers);
		_socketRequestMap = BuildMap(p => p.SocketRequests);
	}

	public PluginCollection(IEnumerable<Plugin> plugins, Config config) : this(PluginsToAssembly(plugins), config) { }

	public IEnumerable<Plugin> AllPlugins() =>
		from weakPlugin in _pluginMap
		let plugin = weakPlugin.Value.WeakToNullable()
		where plugin != null
		select plugin.Instance;

	public Plugin? SpecificPlugin(string name) => _pluginMap.GetValueOrDefault(name)?.WeakToNullable()?.Instance;

	public IEnumerable<T> PluginsAssignableTo<T>() where T : class =>
		from weakPlugin in _pluginMap
		let plugin = weakPlugin.Value.WeakToNullable()
		where plugin != null
		let castedPlugin = plugin.Instance as T
		where castedPlugin != null
		select castedPlugin;

	public IEnumerable<Namespaced<Driver>> AllDrivers() => GetAll(_driverMap);

	public IEnumerable<Namespaced<Driver>> DriversByPlugin(string pluginName) => GetByPlugin(_driverMap, pluginName);

	public Namespaced<Driver>? SpecificDriver(NamespacedName name) => GetSpecific(_driverMap, name);

	public NamespacedName DriverName(Driver driver) => FindNameFor(_driverMap, driver);

	public IEnumerable<Namespaced<PluginAction>> AllActions() => GetAll(_actionMap);

	public IEnumerable<Namespaced<PluginAction>> ActionsByPlugin(string pluginName) =>
		GetByPlugin(_actionMap, pluginName);

	public Namespaced<PluginAction>? SpecificAction(NamespacedName name) => GetSpecific(_actionMap, name);

	public NamespacedName ActionName(PluginAction action) => FindNameFor(_actionMap, action);

	public IEnumerable<Namespaced<Renderer>> AllRenderers() => GetAll(_rendererMap);

	public IEnumerable<Namespaced<Renderer>> RenderersByPlugin(string pluginName) =>
		GetByPlugin(_rendererMap, pluginName);

	public Namespaced<Renderer>? SpecificRenderer(NamespacedName name) => GetSpecific(_rendererMap, name);

	public NamespacedName RendererName(Renderer renderer) => FindNameFor(_rendererMap, renderer);

	public Namespaced<Renderer>? DefaultRenderer() =>
		GetSpecific(_rendererMap, _config.DefaultRenderer)
		?? GetSpecific(_rendererMap, Config.DefaultRendererName);

	public IEnumerable<Namespaced<Trigger>> AllTriggers() => GetAll(_triggerMap);

	public IEnumerable<Namespaced<Trigger>> TriggersByPlugin(string pluginName) => GetByPlugin(_triggerMap, pluginName);

	public Namespaced<Trigger>? SpecificTrigger(NamespacedName name) => GetSpecific(_triggerMap, name);

	public NamespacedName TriggerName(Trigger trigger) => FindNameFor(_triggerMap, trigger);

	public IEnumerable<Namespaced<SocketRequest>> AllSocketRequests() => GetAll(_socketRequestMap);

	public IEnumerable<Namespaced<SocketRequest>> SocketRequestsByPlugin(string pluginName) =>
		GetByPlugin(_socketRequestMap, pluginName);

	public Namespaced<SocketRequest>? SpecificSocketRequest(NamespacedName name) =>
		GetSpecific(_socketRequestMap, name);

	public NamespacedName SocketRequestName(SocketRequest socketRequest) =>
		FindNameFor(_socketRequestMap, socketRequest);

	private static PluginAssembly[] PluginsToAssembly(IEnumerable<Plugin> plugins) {
		var context = new PluginLoadContext("");
		var pluginAssembly = new PluginAssembly(context, plugins.Select(p => new WrappedPlugin(p, context)).ToArray());
		return new[] { pluginAssembly };
	}

	private ConcurrentDictionary<NamespacedName, WeakReference<Namespaced<T>>> BuildMap<T>(
		Func<WrappedPlugin, IEnumerable<Namespaced<T>>> accessor
	) where T : class {
		return new ConcurrentDictionary<NamespacedName, WeakReference<Namespaced<T>>>(
			Plugins
				.SelectMany(a => a.Plugins)
				.SelectMany(accessor)
				.ToDictionary(x => x.NamespacedName, x => new WeakReference<Namespaced<T>>(x))
		);
	}

	private static void AddNamespacedToDict<T>(ConcurrentDictionary<NamespacedName, WeakReference<Namespaced<T>>> dict,
		IEnumerable<Namespaced<T>> enumerable
	) where T : class {
		foreach (var x in enumerable) dict.TryAdd(x.NamespacedName, new WeakReference<Namespaced<T>>(x));
	}

	public Task AddPlugin(IStreamduck streamduck, PluginAssembly assembly) {
		Plugins.Add(assembly);

		foreach (var plugin in assembly.Plugins) {
			AddNamespacedToDict(_driverMap, plugin.Drivers);
			AddNamespacedToDict(_actionMap, plugin.Actions);
		}

		return streamduck.InvokeNewPluginsLoaded(assembly.Plugins.Select(w => w.Instance).ToArray());
	}

	private static IEnumerable<T> GetAll<T>(
		ConcurrentDictionary<NamespacedName, WeakReference<T>> dict
	) where T : class =>
		from weak in dict
		let strong = weak.Value.WeakToNullable()
		where strong != null
		select strong;

	private static IEnumerable<T> GetByPlugin<T>(
		ConcurrentDictionary<NamespacedName, WeakReference<T>> dict,
		string pluginName
	) where T : class =>
		from weak in dict
		where weak.Key.PluginName == pluginName
		let strong = weak.Value.WeakToNullable()
		where strong != null
		select strong;

	private static T? GetSpecific<T>(
		ConcurrentDictionary<NamespacedName, WeakReference<T>> dict,
		NamespacedName name
	) where T : class =>
		dict.GetValueOrDefault(name)?.WeakToNullable();

	public IEnumerable<WrappedPlugin> AllWrappedPlugins() =>
		from weakPlugin in _pluginMap
		let plugin = weakPlugin.Value.WeakToNullable()
		where plugin != null
		select plugin;

	public NamespacedName FindNameFor<T>(ConcurrentDictionary<NamespacedName, WeakReference<Namespaced<T>>> dict,
		T instance
	)
		where T : class =>
	(
		from weak in dict
		let strongValue = weak.Value.WeakToNullable()
		where strongValue != null
		where strongValue.Instance == instance
		select weak.Key
	).First();
}