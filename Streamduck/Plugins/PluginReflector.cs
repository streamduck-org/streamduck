using System;
using System.Collections.Generic;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Streamduck.Attributes;
using Streamduck.Plugins.Methods;
using Streamduck.Utils;

namespace Streamduck.Plugins; 

/**
 * Class that analyzes plugins for things to run
 */
public static class PluginReflector {
	public const BindingFlags Flags = BindingFlags.Public | BindingFlags.NonPublic | BindingFlags.Instance;

	public static IEnumerable<MethodInfo> GetMethods(object obj) => obj.GetType().GetMethods(Flags);
	
	// TODO: Validate plugins for incorrect usages of PluginMethod, report them as warnings during plugin load
	
	public static IEnumerable<PluginAction> AnalyzeActions(IEnumerable<MethodInfo> methods, object obj) =>
		from method in methods 
		where method.GetCustomAttribute<PluginMethodAttribute>() != null 
		where method.ReturnType == typeof(Task) 
		let parameters = method.GetParameters()
		let paramType = parameters.Length == 1 ? parameters[0]?.ParameterType : null
		let isConfigurable = paramType is not null && paramType.IsClass && !paramType.IsAbstract && paramType.GetConstructor(Type.EmptyTypes) is not null
		let isParameterless = parameters.Length <= 0
		where isConfigurable || isParameterless
		select isConfigurable ?
			GetGenericConfigurableAction(parameters[0].ParameterType, method, obj) :
			new ReflectedAction(
				method.GetCustomAttribute<NameAttribute>()?.Name ?? method.Name.FormatAsWords(), 
				() => {
					var task = (Task?)method.Invoke(obj, Flags, null, null, null);
					return task ?? Task.CompletedTask;
				},
				method.GetCustomAttribute<DescriptionAttribute>()?.Description
			);
	
	private static PluginAction GetGenericConfigurableAction(Type configType, MethodInfo method, object obj) {
		return (PluginAction)ConfigurableActionMethod.MakeGenericMethod(configType).Invoke(null, ConfigurableActionFlags, null, new[] { method, obj },
			null)!;
	}

	private const BindingFlags ConfigurableActionFlags = BindingFlags.Static | BindingFlags.NonPublic;
	private static readonly MethodInfo ConfigurableActionMethod =
		typeof(PluginReflector).GetMethod(nameof(GetConfigurableAction), ConfigurableActionFlags)!;

	private static ConfigurableReflectedAction<T> GetConfigurableAction<T>(MethodBase method, object obj) where T : class, new() => new(
			method.GetCustomAttribute<NameAttribute>()?.Name ?? method.Name.FormatAsWords(),
			options => {
				var task = (Task?)method.Invoke(obj, Flags, null, new object?[] { options }, null);
				return task ?? Task.CompletedTask;
			},
			method.GetCustomAttribute<DescriptionAttribute>()?.Description
		);
}