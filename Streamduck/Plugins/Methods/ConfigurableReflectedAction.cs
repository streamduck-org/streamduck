using System;
using System.Threading.Tasks;
using Streamduck.Interfaces;

namespace Streamduck.Plugins.Methods; 

public class ConfigurableReflectedAction<T> (string name, Func<T?, Task> actionToCall,
		string? description = null)
	: PluginAction, IConfigurable<T> where T : class, new() {
	public override string Name { get; } = name;
	public override string? Description { get; } = description;

	public override Task Invoke() => actionToCall.Invoke(Options);
	public T Options { get; set; } = new();
}