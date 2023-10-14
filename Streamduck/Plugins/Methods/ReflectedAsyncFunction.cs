using System;
using System.Threading.Tasks;
using Streamduck.Scripting;

namespace Streamduck.Plugins.Methods; 

public class ReflectedAsyncFunction : AsyncPluginFunction {
	private readonly Func<object[], Task<object[]>> _functionToCall;
	
	public override string Name { get; }
	public override string? Description { get; }
	public override DataInfo[] Parameters { get; }
	
	public override DataInfo[] Returns { get; }

	public override Task<object[]> Invoke(object[] arguments) => _functionToCall.Invoke(arguments);

	public ReflectedAsyncFunction(string name, DataInfo[] parameters, DataInfo returnInfo, 
		Func<object[], Task<object[]>> functionToCall, string? description = null) {
		Name = name;
		Parameters = parameters;
		Returns = new[] { returnInfo }; 
		_functionToCall = functionToCall;
		Description = description;
	}
}