// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Text.Json;
using System.Threading.Tasks;
using NLog;
using Streamduck.Interfaces;
using Streamduck.Plugins;

namespace Streamduck.Configuration;

public static class GlobalConfig {
	private const BindingFlags StaticNonPublic = BindingFlags.Static | BindingFlags.NonPublic;
	private static readonly Logger L = LogManager.GetCurrentClassLogger();

	public static readonly string GlobalConfigFolder = Path.Join(
		Config.StreamduckFolder,
		"global"
	);

	private static readonly MethodInfo LoadGenericIConfigurableMethod =
		typeof(GlobalConfig).GetMethod(nameof(LoadGenericIConfigurable), StaticNonPublic)!;

	private static readonly MethodInfo SaveGenericIConfigurableMethod =
		typeof(GlobalConfig).GetMethod(nameof(SaveGenericIConfigurable), StaticNonPublic)!;

	private static string PluginFolderPath(WrappedPlugin plugin) => Path.Join(
		GlobalConfigFolder,
		plugin.Name
	);

	private static string PluginFilePath(string pluginFolderPath) => Path.Join(
		pluginFolderPath,
		"config.json"
	);

	public static async Task LoadPlugin(WrappedPlugin plugin) {
		var pluginFolderPath = PluginFolderPath(plugin);

		// Load plugin config if exists
		var pluginConfigurableType = plugin.Instance.GetType().GetInterfaces().FirstOrDefault(x =>
			x.IsGenericType && x.GetGenericTypeDefinition() == typeof(IConfigurable<>));

		if (pluginConfigurableType is not null) {
			var pluginFilePath = PluginFilePath(pluginFolderPath);

			await (Task)LoadGenericIConfigurableMethod.MakeGenericMethod(pluginConfigurableType.GenericTypeArguments[0])
				.Invoke(null, [plugin.Instance, pluginFilePath])!;
		}

		await LoadEnumerable(
			plugin.Actions.Select(x => x.Instance),
			pluginFolderPath,
			"actions"
		);

		await LoadEnumerable(
			plugin.Drivers.Select(x => x.Instance),
			pluginFolderPath,
			"drivers"
		);

		await LoadEnumerable(
			plugin.Renderers.Select(x => x.Instance),
			pluginFolderPath,
			"renderers"
		);

		await LoadEnumerable(
			plugin.Triggers.Select(x => x.Instance),
			pluginFolderPath,
			"triggers"
		);
	}

	private static async Task LoadEnumerable<T>(IEnumerable<T> iter, string basePath, string name) where T : INamed {
		var collectionPath = Path.Join(
			basePath,
			name
		);

		foreach (var item in iter) {
			var configurableType = item.GetType().GetInterfaces().FirstOrDefault(x =>
				x.IsGenericType && x.GetGenericTypeDefinition() == typeof(IConfigurable<>));

			if (configurableType is null) continue; // Only allow items that have generic IConfigurable

			var filePath = Path.Join(collectionPath, $"{item.Name}.json");

			await (Task)LoadGenericIConfigurableMethod.MakeGenericMethod(configurableType.GenericTypeArguments[0])
				.Invoke(null, [item, filePath])!;
		}
	}

	private static async Task LoadGenericIConfigurable<T>(IConfigurable<T> obj, string filePath)
		where T : class, new() {
		if (!File.Exists(filePath)) return;

		try {
			var data = await File.ReadAllBytesAsync(filePath);
			using var buffer = new MemoryStream(data);
			var config = await JsonSerializer.DeserializeAsync<T>(buffer);

			if (config is null) {
				L.Error($"Couldn't properly cast data from '{filePath}' to '{typeof(T).FullName}'");
				return;
			}

			obj.Config = config;
		} catch (Exception e) {
			L.Error($"Failed to load config at '{filePath}': {e}");
		}
	}

	public static async Task SavePlugin(WrappedPlugin plugin) {
		var pluginFolderPath = PluginFolderPath(plugin);

		// Save plugin config if exists
		var pluginConfigurableType = plugin.Instance.GetType().GetInterfaces().FirstOrDefault(x =>
			x.IsGenericType && x.GetGenericTypeDefinition() == typeof(IConfigurable<>));

		if (pluginConfigurableType is not null) {
			var pluginFilePath = PluginFilePath(pluginFolderPath);

			try {
				Directory.CreateDirectory(pluginFolderPath);
			} catch (Exception e) {
				L.Error("Error happened while trying to create folders for config {0}", e);
				return;
			}

			await (Task)SaveGenericIConfigurableMethod.MakeGenericMethod(pluginConfigurableType.GenericTypeArguments[0])
				.Invoke(null, [plugin.Instance, pluginFilePath])!;
		}

		// Save its derivatives
		await SaveEnumerable(
			plugin.Actions.Select(x => x.Instance),
			pluginFolderPath,
			"actions"
		);

		await SaveEnumerable(
			plugin.Drivers.Select(x => x.Instance),
			pluginFolderPath,
			"drivers"
		);

		await SaveEnumerable(
			plugin.Renderers.Select(x => x.Instance),
			pluginFolderPath,
			"renderers"
		);

		await SaveEnumerable(
			plugin.Triggers.Select(x => x.Instance),
			pluginFolderPath,
			"triggers"
		);
	}

	private static async Task SaveEnumerable<T>(IEnumerable<T> iter, string basePath, string name) where T : INamed {
		var collectionPath = Path.Join(
			basePath,
			name
		);

		var folderCreated = false;

		foreach (var item in iter) {
			var configurableType = item.GetType().GetInterfaces().FirstOrDefault(x =>
				x.IsGenericType && x.GetGenericTypeDefinition() == typeof(IConfigurable<>));

			if (configurableType is null) continue; // Only allow items that have generic IConfigurable

			if (!folderCreated) {
				try {
					Directory.CreateDirectory(collectionPath);
				} catch (Exception e) {
					L.Error("Error happened while trying to create folders for config {0}", e);
					return;
				}

				folderCreated = true;
			}

			var filePath = Path.Join(collectionPath, $"{item.Name}.json");

			await (Task)SaveGenericIConfigurableMethod.MakeGenericMethod(configurableType.GenericTypeArguments[0])
				.Invoke(null, [item, filePath])!;
		}
	}

	private static async Task SaveGenericIConfigurable<T>(IConfigurable<T> obj, string filePath)
		where T : class, new() {
		using var buffer = new MemoryStream();

		await JsonSerializer.SerializeAsync(
			buffer,
			obj.Config
		);

		await File.WriteAllBytesAsync(filePath, buffer.ToArray());
	}
}