// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System;
using System.Collections.Generic;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Streamduck.Actions;
using Streamduck.Attributes;
using Streamduck.Plugins.Methods;
using Streamduck.Utils;

namespace Streamduck.Plugins;

/**
 * Class that analyzes plugins for things to run
 */
public static class PluginReflector {
	public const BindingFlags Flags = BindingFlags.Public | BindingFlags.NonPublic | BindingFlags.Instance;

	private const BindingFlags ActionFlags = BindingFlags.Static | BindingFlags.NonPublic;

	private const BindingFlags ConfigurableActionFlags = BindingFlags.Static | BindingFlags.NonPublic;

	private static readonly MethodInfo ActionMethod =
		typeof(PluginReflector).GetMethod(nameof(GetAction), ActionFlags)!;

	private static readonly MethodInfo ConfigurableActionMethod =
		typeof(PluginReflector).GetMethod(nameof(GetConfigurableAction), ConfigurableActionFlags)!;

	public static IEnumerable<MethodInfo> GetMethods(object obj) => obj.GetType().GetMethods(Flags);

	// TODO: Validate plugins for incorrect usages of PluginMethod, report them as warnings during plugin load

	public static IEnumerable<PluginAction> AnalyzeActions(IEnumerable<MethodInfo> methods, object obj) =>
		from method in methods
		where method.GetCustomAttribute<PluginMethodAttribute>() != null
		where method.ReturnType == typeof(Task)
		let parameters = method.GetParameters()
		let paramType = parameters.Length is 1 or 2 ? parameters[0]?.ParameterType : null
		let configType = parameters.Length == 2 ? parameters[1]?.ParameterType : null
		let isParameterful = paramType is not null && paramType.IsClass && !paramType.IsAbstract &&
		                     paramType.GetConstructor(Type.EmptyTypes) is not null
		let isConfigurable = configType is not null && configType.IsClass && !configType.IsAbstract &&
		                     configType.GetConstructor(Type.EmptyTypes) is not null
		let isParameterless = parameters.Length <= 0
		where isConfigurable || isParameterless || isParameterful
		select isConfigurable && isParameterful ? GetGenericConfigurableAction(paramType, configType, method, obj) :
			isParameterful ? GetGenericAction(paramType, method, obj) :
			new ReflectedAction<object>(
				method.GetCustomAttribute<NameAttribute>()?.Name ?? method.Name.FormatAsWords(),
				_ => {
					var task = (Task?)method.Invoke(obj, Flags, null, null, null);
					return task ?? Task.CompletedTask;
				},
				method.GetCustomAttribute<DescriptionAttribute>()?.Description
			);

	private static PluginAction GetGenericAction(Type paramType, MethodInfo method, object obj) {
		return (PluginAction)ActionMethod.MakeGenericMethod(paramType)
			.Invoke(null, ActionFlags, null, new[] { method, obj },
				null)!;
	}

	private static ReflectedAction<T> GetAction<T>(MethodBase method, object obj) where T : class, new() => new(
		method.GetCustomAttribute<NameAttribute>()?.Name ?? method.Name.FormatAsWords(),
		options => {
			var task = (Task?)method.Invoke(obj, Flags, null, new object?[] { options }, null);
			return task ?? Task.CompletedTask;
		},
		method.GetCustomAttribute<DescriptionAttribute>()?.Description
	);

	private static PluginAction GetGenericConfigurableAction(Type paramType, Type configType, MethodInfo method,
		object obj) {
		return (PluginAction)ConfigurableActionMethod.MakeGenericMethod(paramType, configType)
			.Invoke(null, ConfigurableActionFlags, null, new[] { method, obj },
				null)!;
	}

	private static ConfigurableReflectedAction<T, C> GetConfigurableAction<T, C>(MethodBase method, object obj)
		where T : class, new() where C : class, new() => new(
		method.GetCustomAttribute<NameAttribute>()?.Name ?? method.Name.FormatAsWords(),
		(options, config) => {
			var task = (Task?)method.Invoke(obj, Flags, null, new object?[] { options, config }, null);
			return task ?? Task.CompletedTask;
		},
		method.GetCustomAttribute<DescriptionAttribute>()?.Description
	);

	public static IEnumerable<T> GetPluginTypes<T>(Type pluginType, bool useEmptyOnes) where T : class {
		foreach (var type in pluginType.Assembly.GetTypes()) {
			if (!type.IsAssignableTo(typeof(T))) continue;

			if (type.GetCustomAttribute<AutoAddAttribute>() is not { } attribute) continue;
			if ((attribute.PluginClass is not null || !useEmptyOnes) && attribute.PluginClass != pluginType) continue;

			if (type.GetConstructor(Type.EmptyTypes) is not { } constructor) continue;
			yield return (T)constructor.Invoke([]);
		}
	}
}