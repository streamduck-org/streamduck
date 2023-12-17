using System;
using System.Reflection;
using System.Runtime.Loader;

namespace Streamduck.Plugins.Loaders;

public class PluginLoadContext(string pluginPath) : AssemblyLoadContext(true) {
	private readonly AssemblyDependencyResolver _resolver = new(pluginPath);

	protected override Assembly? Load(AssemblyName assemblyName) {
		var assemblyPath = _resolver.ResolveAssemblyToPath(assemblyName);
		return assemblyPath == null ? null : LoadFromAssemblyPath(assemblyPath);
	}

	protected override IntPtr LoadUnmanagedDll(string unmanagedDllName) {
		var libraryPath = _resolver.ResolveUnmanagedDllToPath(unmanagedDllName);
		return libraryPath != null ? LoadUnmanagedDllFromPath(libraryPath) : IntPtr.Zero;
	}
}