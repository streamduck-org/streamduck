// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System.Linq;
using System.Threading.Tasks;

namespace Streamduck.Plugins.Extensions;

public static class StreamduckExtensions {
	public static Task InvokePluginsLoaded(this IStreamduck collection) =>
		Task.WhenAll(collection.Plugins.AllPlugins().Select(p => p.OnPluginsLoaded(collection)));

	public static Task InvokeNewPluginsLoaded(this IStreamduck collection, Plugin[] plugins) =>
		Task.WhenAll(collection.Plugins.AllPlugins().Select(p => p.OnNewPluginsLoaded(plugins, collection)));
}