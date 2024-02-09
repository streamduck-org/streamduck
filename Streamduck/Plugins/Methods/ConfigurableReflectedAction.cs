using System;
using System.Threading.Tasks;
using Streamduck.Interfaces;

namespace Streamduck.Plugins.Methods; 

public class ConfigurableReflectedAction<T, C> (string name, Func<T, C, Task> actionToCall,
		string? description = null)
	: PluginAction<T>, IConfigurable<C> where T : class, new() where C : class, new() {
	public override string Name { get; } = name;
	public override string? Description { get; } = description;
	public override Task Invoke(T data) => actionToCall.Invoke(data, Config);
	public C Config { get; set; } = new();
}