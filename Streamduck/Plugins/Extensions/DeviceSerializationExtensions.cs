// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System.Linq;
using System.Reflection;
using System.Text.Json;
using Streamduck.Actions;
using Streamduck.Configuration.Devices;
using Streamduck.Cores;
using Streamduck.Triggers;

namespace Streamduck.Plugins.Extensions;

public static class DeviceSerializationExtensions {
	private const BindingFlags StaticNonPublic = BindingFlags.Static | BindingFlags.NonPublic;

	private static readonly MethodInfo SerializeGenericTrigger =
		typeof(DeviceSerializationExtensions).GetMethod(nameof(SerializeGenericTriggerOptions), StaticNonPublic)!;

	public static SerializedAction Serialize(this ActionInstance actionInstance, IPluginQuery pluginQuery) =>
		new() {
			Action = pluginQuery.ActionName(actionInstance.Original),
			Data = JsonSerializer.SerializeToElement(actionInstance.Data)
		};

	public static SerializedTrigger Serialize(this TriggerInstance triggerInstance, IPluginQuery pluginQuery) =>
		new() {
			Trigger = pluginQuery.TriggerName(triggerInstance.Original),
			Actions = triggerInstance.Actions.Select(a => a.Serialize(pluginQuery)).ToArray(),
			Data = SerializeTriggerOptions(triggerInstance)
		};

	private static JsonElement? SerializeTriggerOptions(TriggerInstance instance) {
		var potentialBaseType = instance.GetType().BaseType;
		if (potentialBaseType is null) return null;
		if (!potentialBaseType.IsGenericType) return null;
		if (potentialBaseType.GetGenericTypeDefinition() != typeof(TriggerInstance<>)) return null;
		return (JsonElement?) SerializeGenericTrigger.MakeGenericMethod(potentialBaseType.GenericTypeArguments)
			.Invoke(null, [instance]);
	}

	private static JsonElement SerializeGenericTriggerOptions<T>(TriggerInstance<T> instance) where T : class, new() =>
		JsonSerializer.SerializeToElement(instance.Options);
	
	public static SerializedScreenItem Serialize(this ScreenItem screenItem, IPluginQuery pluginQuery) {
		var triggers = screenItem.Triggers.Select(t => t.Serialize(pluginQuery)).ToArray();

		if (screenItem is ScreenItem.IRenderable renderable)
			return new SerializedScreenItem {
				Triggers = triggers,
				RendererName = renderable.RendererName,
				RendererSettings = renderable.RendererSettings is not null
					? JsonSerializer.SerializeToElement(renderable.RendererSettings)
					: null
			};
		
		return new SerializedScreenItem {
			Triggers = triggers
		};
	}

	public static SerializedScreen Serialize(this Screen screen, IPluginQuery pluginQuery) => new() {
		Items = screen.Items.Select(i => i?.Serialize(pluginQuery)).ToArray()
	};
}