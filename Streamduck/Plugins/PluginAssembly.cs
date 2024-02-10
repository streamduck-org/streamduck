// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System.Collections.Generic;
using Streamduck.Plugins.Loaders;

namespace Streamduck.Plugins;

/**
 * Collection of all types inside of a plugin
 */
public class PluginAssembly(PluginLoadContext context, WrappedPlugin[] plugins) {
	internal readonly PluginLoadContext Context = context;

	public IEnumerable<WrappedPlugin> Plugins => plugins;

	public void Unload() {
		Context.Unload();
	}
}