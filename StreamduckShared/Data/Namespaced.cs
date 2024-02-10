// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using Streamduck.Plugins;

namespace Streamduck.Data;

/**
 * Type that belongs to a plugin
 */
public class Namespaced<T>(NamespacedName namespacedName, T instance)
	where T : class {
	public NamespacedName NamespacedName { get; } = namespacedName;
	public T Instance { get; } = instance;

	public string Name => NamespacedName.Name;
	public string PluginName => NamespacedName.PluginName;
}