// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System.Collections.Generic;
using System.Linq;
using Streamduck.Data;
using Streamduck.Interfaces;
using Streamduck.Plugins.Loaders;
using Streamduck.Rendering;
using Streamduck.Triggers;

namespace Streamduck.Plugins;

/**
 * Plugin wrapper that includes namespaced versions of all plugin types
 */
public sealed class WrappedPlugin {
	private readonly PluginLoadContext _originatedFrom;

	public WrappedPlugin(Plugin instance, PluginLoadContext originatedFrom) {
		Instance = instance;
		_originatedFrom = originatedFrom;

		Name = Instance.Name;
		Drivers = Instance.Drivers
			.Select(Namespace)
			.ToArray();
		var methods = PluginReflector.GetMethods(instance).ToArray();
		Actions = Instance.Actions
			.Concat(PluginReflector.AnalyzeActions(methods, instance))
			.Select(Namespace)
			.ToArray();
		Renderers = Instance.Renderers
			.Select(Namespace)
			.ToArray();
		Triggers = Instance.Triggers
			.Select(Namespace)
			.ToArray();
	}

	public Plugin Instance { get; }

	public string Name { get; }

	public IEnumerable<Namespaced<Driver>> Drivers { get; }
	public IEnumerable<Namespaced<PluginAction>> Actions { get; }
	public IEnumerable<Namespaced<Renderer>> Renderers { get; }
	public IEnumerable<Namespaced<Trigger>> Triggers { get; }

	public Namespaced<T> Namespace<T>(T instance) where T : class, INamed =>
		new(new NamespacedName(Name, instance.Name), instance);

	public bool BelongsTo(PluginAssembly assembly) => assembly.Context.Equals(_originatedFrom);
}