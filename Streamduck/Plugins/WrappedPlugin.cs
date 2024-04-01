// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System.Collections.Generic;
using System.Linq;
using Streamduck.Actions;
using Streamduck.Data;
using Streamduck.Interfaces;
using Streamduck.Plugins.Loaders;
using Streamduck.Rendering;
using Streamduck.Socket;
using Streamduck.Triggers;

namespace Streamduck.Plugins;

/**
 * Plugin wrapper that includes namespaced versions of all plugin types
 */
public sealed class WrappedPlugin {
	private readonly PluginLoadContext _originatedFrom;

	public WrappedPlugin(Plugin instance, PluginLoadContext originatedFrom, bool isFirst = false) {
		Instance = instance;
		_originatedFrom = originatedFrom;

		Name = Instance.Name;
		Drivers = Instance.Drivers
			.Concat(PluginReflector.GetPluginTypes<Driver>(instance.GetType(), isFirst))
			.Select(Namespace)
			.ToArray();
		var methods = PluginReflector.GetMethods(instance).ToArray();
		Actions = Instance.Actions
			.Concat(PluginReflector.AnalyzeActions(methods, instance))
			.Concat(PluginReflector.GetPluginTypes<PluginAction>(instance.GetType(), isFirst))
			.Select(Namespace)
			.ToArray();
		Renderers = Instance.Renderers
			.Concat(PluginReflector.GetPluginTypes<Renderer>(instance.GetType(), isFirst))
			.Select(Namespace)
			.ToArray();
		Triggers = Instance.Triggers
			.Concat(PluginReflector.GetPluginTypes<Trigger>(instance.GetType(), isFirst))
			.Select(Namespace)
			.ToArray();
		SocketRequests = Instance.SocketRequests
			.Concat(PluginReflector.GetPluginTypes<SocketRequest>(instance.GetType(), isFirst))
			.Select(Namespace)
			.ToArray();
	}

	public Plugin Instance { get; }

	public string Name { get; }

	public IEnumerable<Namespaced<Driver>> Drivers { get; }
	public IEnumerable<Namespaced<PluginAction>> Actions { get; }
	public IEnumerable<Namespaced<Renderer>> Renderers { get; }
	public IEnumerable<Namespaced<Trigger>> Triggers { get; }
	public IEnumerable<Namespaced<SocketRequest>> SocketRequests { get; }

	public Namespaced<T> Namespace<T>(T instance) where T : class, INamed =>
		new(new NamespacedName(Name, instance.Name), instance);

	public bool BelongsTo(PluginAssembly assembly) => assembly.Context.Equals(_originatedFrom);
}