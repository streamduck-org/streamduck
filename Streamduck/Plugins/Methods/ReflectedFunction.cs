using System;
using Streamduck.Scripting;

namespace Streamduck.Plugins.Methods; 

public class ReflectedFunction : PluginFunction {
	private readonly Func<object[], object[]> _functionToCall;
	
	public override string Name { get; }
	public override string? Description { get; }
	public override DataInfo[] Parameters { get; }
	
	public override DataInfo[] Returns { get; }

	public override object[] Invoke(object[] arguments) => _functionToCall.Invoke(arguments);

	public ReflectedFunction(string name, DataInfo[] parameters, DataInfo returnInfo, 
		Func<object[], object[]> functionToCall, string? description = null) {
		Name = name;
		Parameters = parameters;
		Returns = new[] { returnInfo }; 
		_functionToCall = functionToCall;
		Description = description;
	}
}