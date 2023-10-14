using System;
using System.Collections.Generic;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Streamduck.Attributes;
using Streamduck.Plugins.Methods;
using Streamduck.Scripting;
using Streamduck.Utils;

namespace Streamduck.Plugins; 

/**
 * Class that analyzes plugins for things to run
 */
public static class PluginReflector {
	public const BindingFlags Flags = BindingFlags.Public | BindingFlags.NonPublic | BindingFlags.Instance;

	public static IEnumerable<MethodInfo> GetMethods(object obj) => obj.GetType().GetMethods(Flags);

	public static IEnumerable<PluginAction> AnalyzeActions(IEnumerable<MethodInfo> methods, object obj) =>
		from method in methods 
		where method.GetCustomAttribute<PluginMethodAttribute>() != null 
		where method.ReturnType == typeof(void) 
		select new ReflectedAction(
			method.GetCustomAttribute<NameAttribute>()?.Name ?? method.Name.FormatAsWords(),
			ParseParameters(method).ToArray(),
			args => method.Invoke(obj, Flags, null, args, null),
			method.GetCustomAttribute<DescriptionAttribute>()?.Description
		);
	
	public static IEnumerable<PluginFunction> AnalyzeFunctions(IEnumerable<MethodInfo> methods, object obj) =>
		from method in methods 
		where method.GetCustomAttribute<PluginMethodAttribute>() != null 
		where method.ReturnType != typeof(void) 
		where method.ReturnType != typeof(Task)
		where !(method.ReturnType.IsGenericType && method.ReturnType.GetGenericTypeDefinition() == typeof(Task<>))  
		select new ReflectedFunction(
			method.GetCustomAttribute<NameAttribute>()?.Name ?? method.Name.FormatAsWords(),
			ParseParameters(method).ToArray(),
			new DataInfo(
				method.ReturnType,
				method.ReturnParameter.GetCustomAttribute<NameAttribute>()?.Name ?? "Out" 
			) {
				Description = method.ReturnParameter.GetCustomAttribute<DescriptionAttribute>()?.Description
			},
			args => new[] {method.Invoke(obj, Flags, null, args, null)},
			method.GetCustomAttribute<DescriptionAttribute>()?.Description
		);
	
	public static IEnumerable<AsyncPluginAction> AnalyzeAsyncActions(IEnumerable<MethodInfo> methods, object obj) =>
		from method in methods 
		where method.GetCustomAttribute<PluginMethodAttribute>() != null 
		where method.ReturnType == typeof(Task) 
		select new ReflectedAsyncAction(
			method.GetCustomAttribute<NameAttribute>()?.Name ?? method.Name.FormatAsWords(),
			ParseParameters(method).ToArray(),
			async args => {
				var task = (Task?)method.Invoke(obj, Flags, null, args, null);
				if (task == null) return;
				await task.ConfigureAwait(false);
			},
			method.GetCustomAttribute<DescriptionAttribute>()?.Description
		);
	
	public static IEnumerable<AsyncPluginFunction> AnalyzeAsyncFunctions(IEnumerable<MethodInfo> methods, object obj) =>
		from method in methods 
		where method.GetCustomAttribute<PluginMethodAttribute>() != null 
		where method.ReturnType != typeof(void) 
		where method.ReturnType.IsGenericType && method.ReturnType.GetGenericTypeDefinition() == typeof(Task<>)  
		select new ReflectedAsyncFunction(
			method.GetCustomAttribute<NameAttribute>()?.Name ?? method.Name.FormatAsWords(),
			ParseParameters(method).ToArray(),
			new DataInfo(
				method.ReturnType.GenericTypeArguments[0],
				method.ReturnParameter.GetCustomAttribute<NameAttribute>()?.Name ?? "Out" 
			) {
				Description = method.ReturnParameter.GetCustomAttribute<DescriptionAttribute>()?.Description
			},
			async args => {
				var task = (Task?)method.Invoke(obj, Flags, null, args, null);
				if (task == null) return Array.Empty<object>();
				await task.ConfigureAwait(false);
				return new[] { (object)((dynamic)task).Result };
			},
			method.GetCustomAttribute<DescriptionAttribute>()?.Description
		);

	private static IEnumerable<DataInfo> ParseParameters(MethodBase method) {
		return method.GetParameters().Select(parameter => new DataInfo(
			parameter.ParameterType,
			parameter.GetCustomAttribute<NameAttribute>()?.Name ?? parameter.Name?.FormatAsWords() ?? "In"
		) {
			Description = parameter.GetCustomAttribute<DescriptionAttribute>()?.Description
		});
	}
}