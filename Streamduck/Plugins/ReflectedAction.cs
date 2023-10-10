using System;
using Streamduck.Scripting;

namespace Streamduck.Plugins; 

public class ReflectedAction : PluginAction {
	private readonly Action<object[]> _actionToCall;
	
	public override string Name { get; }
	public override string? Description { get; }
	public override DataInfo[] Parameters { get; }
	
	public override void Invoke(object[] arguments) {
		_actionToCall.Invoke(arguments);
	}

	public ReflectedAction(string name, DataInfo[] parameters, Action<object[]> actionToCall,
		string? description = null) {
		Name = name;
		Parameters = parameters;
		_actionToCall = actionToCall;
		Description = description;
	}
}