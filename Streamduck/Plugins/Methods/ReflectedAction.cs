using System;
using System.Threading.Tasks;

namespace Streamduck.Plugins.Methods; 

public class ReflectedAction(string name, Func<Task> actionToCall,
		string? description = null)
	: PluginAction {
	public override string Name { get; } = name;
	public override string? Description { get; } = description;

	public override Task Invoke() => actionToCall.Invoke();
}