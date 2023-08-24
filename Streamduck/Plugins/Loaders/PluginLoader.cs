using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using NLog;

namespace Streamduck.Plugins.Loaders;

public static class PluginLoader {
	private static readonly Logger L = LogManager.GetCurrentClassLogger();

	public static IEnumerable<WrappedPlugin> Load(string path) {
		var curDir = Directory.GetCurrentDirectory();
		var fullPath = Path.Combine(curDir, path);

		try {
			var plugin = CreatePlugins(LoadPlugin(fullPath));
			return plugin;
		} catch (Exception e) {
			L.Error("Failed to load plugin at {0}\nReason: {1}", path, e);
			return Array.Empty<WrappedPlugin>();
		}
	}

	public static IEnumerable<WrappedPlugin> LoadFromFolder(string pathToFolder) {
		L.Info("Loading plugins in {0} folder...", pathToFolder);

		var curDir = Directory.GetCurrentDirectory();
		var fullPath = Path.Combine(curDir, pathToFolder);

		foreach (var filePath in Directory.GetFiles(pathToFolder)) {
			if (!filePath.EndsWith("dll")) continue;

			foreach (var plugin in Load(filePath)) yield return plugin;
		}

		foreach (var directory in Directory.GetDirectories(pathToFolder)) {
			var directoryName = Path.GetFileName(directory);

			var dllPath = Path.Combine(directory, $"{directoryName}.dll");

			foreach (var plugin in Load(dllPath)) yield return plugin;
		}
	}

	private static Assembly LoadPlugin(string assemblyPath) {
		var context = new PluginLoadContext(assemblyPath);
		return context.LoadFromAssemblyName(new AssemblyName(Path.GetFileNameWithoutExtension(assemblyPath)));
	}

	private static IEnumerable<WrappedPlugin> CreatePlugins(Assembly assembly) {
		var loadedPlugins = 0;

		foreach (var type in assembly.GetTypes()) {
			if (!typeof(Plugin).IsAssignableFrom(type)) continue;
			if (Activator.CreateInstance(type) is not Plugin plugin) continue;

			loadedPlugins++;
			var wrapped = new WrappedPlugin(plugin);
			L.Info("Loaded plugin \"{0}\" ({1})", wrapped.Name, assembly.Location);
			yield return wrapped;
		}

		if (loadedPlugins != 0) yield break;

		var types = string.Join(",", assembly.GetTypes().Select(t => t.FullName));
		throw new ApplicationException(
			$"{assembly} ({assembly.Location}) doesn't have any types that implement Plugin class\n" +
			$"Available types: {types}");
	}
}