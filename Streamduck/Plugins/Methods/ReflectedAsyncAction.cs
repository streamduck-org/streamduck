using System;
using System.Threading.Tasks;
using Streamduck.Scripting;

namespace Streamduck.Plugins.Methods; 

public class ReflectedAsyncAction : AsyncPluginAction {
	private readonly Func<object[], Task> _actionToCall;
	
	public override string Name { get; }
	public override string? Description { get; }
	public override DataInfo[] Parameters { get; }
	
	public override Task Invoke(object[] arguments) => _actionToCall.Invoke(arguments);

	public ReflectedAsyncAction(string name, DataInfo[] parameters, Func<object[], Task> actionToCall,
		string? description = null) {
		Name = name;
		Parameters = parameters;
		_actionToCall = actionToCall;
		Description = description;
	}
}