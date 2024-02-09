using System;
using System.Threading.Tasks;

namespace Streamduck.Plugins.Methods; 

public class ReflectedAction<T>(string name, Func<T, Task> actionToCall,
		string? description = null)
	: PluginAction<T> where T : class, new() {
	public override string Name { get; } = name;
	public override string? Description { get; } = description;
	public override Task Invoke(T data) => actionToCall.Invoke(data);
}