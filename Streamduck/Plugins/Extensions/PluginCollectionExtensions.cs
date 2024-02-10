// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System.Linq;
using System.Threading.Tasks;
using Streamduck.Configuration;
using Streamduck.Cores;
using Streamduck.Devices;

namespace Streamduck.Plugins.Extensions;

public static class PluginCollectionExtensions {
	public static Task InvokePluginsLoaded(this PluginCollection collection) =>
		Task.WhenAll(collection.AllPlugins().Select(p => p.OnPluginsLoaded(collection)));

	public static Task InvokeNewPluginsLoaded(this PluginCollection collection, Plugin[] plugins) =>
		Task.WhenAll(collection.AllPlugins().Select(p => p.OnNewPluginsLoaded(plugins, collection)));

	public static Task InvokeDeviceConnected(this PluginCollection collection, NamespacedDeviceIdentifier identifier,
		Core deviceCore) =>
		Task.WhenAll(collection.AllPlugins().Select(p => p.OnDeviceConnected(identifier, deviceCore)));

	public static Task
		InvokeDeviceDisconnected(this PluginCollection collection, NamespacedDeviceIdentifier identifier) =>
		Task.WhenAll(collection.AllPlugins().Select(p => p.OnDeviceDisconnected(identifier)));

	public static Task LoadAllPluginConfigs(this PluginCollection collection) =>
		Task.WhenAll(collection.AllWrappedPlugins().Select(GlobalConfig.LoadPlugin));

	public static Task SaveAllPluginConfigs(this PluginCollection collection) =>
		Task.WhenAll(collection.AllWrappedPlugins().Select(GlobalConfig.SavePlugin));
}