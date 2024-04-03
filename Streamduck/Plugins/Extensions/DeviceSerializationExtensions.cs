// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System;
using System.Linq;
using System.Reflection;
using System.Text.Json;
using System.Threading.Tasks;
using NLog;
using Streamduck.Actions;
using Streamduck.Configuration.Devices;
using Streamduck.Cores;
using Streamduck.Cores.ScreenItems;
using Streamduck.Inputs;
using Streamduck.Rendering;
using Streamduck.Triggers;

namespace Streamduck.Plugins.Extensions;

public static class DeviceSerializationExtensions {
	private static readonly Logger _l = LogManager.GetCurrentClassLogger();
	private const BindingFlags StaticNonPublic = BindingFlags.Static | BindingFlags.NonPublic;

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
		return (JsonElement?) SerializeGenericTriggerMethod.MakeGenericMethod(potentialBaseType.GenericTypeArguments)
			.Invoke(null, [instance]);
	}

	private static readonly MethodInfo SerializeGenericTriggerMethod =
		typeof(DeviceSerializationExtensions).GetMethod(nameof(SerializeGenericTriggerOptions), StaticNonPublic)!;
	
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
		Items = screen.Items.Select(i => i?.Serialize(pluginQuery)).ToArray(),
		CanWrite = screen.CanWrite
	};

	public static DeviceConfig Serialize(this Core core, IPluginQuery pluginQuery) => new() {
		Device = core.DeviceIdentifier,
		ScreenStack = core.ScreenStack.Select(s => s.Serialize(pluginQuery)).ToArray()
	};

	public static ActionInstance? Deserialize(this SerializedAction serializedAction, IPluginQuery pluginQuery) {
		if (pluginQuery.SpecificAction(serializedAction.Action) is not { } action) {
			_l.Warn($"Failed to load action, '{serializedAction.Action}' no longer exists");
			return null;
		}

		var instance = new ActionInstance(action.Instance);

		if (serializedAction.Data is not { } data) return instance;

		var baseType = action.Instance.GetType().BaseType;
		if (baseType is null ||
		    !baseType.IsGenericType || 
		    baseType.GetGenericTypeDefinition() != typeof(PluginAction<>)) return instance;

		instance.Data = data.Deserialize(baseType.GenericTypeArguments[0]);

		return instance;
	}

	public static async Task<TriggerInstance?> Deserialize(this SerializedTrigger serializedTrigger, IPluginQuery pluginQuery) {
		if (pluginQuery.SpecificTrigger(serializedTrigger.Trigger) is not { } trigger) {
			_l.Warn($"Failed to load trigger, '{serializedTrigger.Trigger}' no longer exists");
			return null;
		}

		var instance = await trigger.Instance.CreateInstance();
		
		instance.AddActions(serializedTrigger.Actions.Select(sA => sA.Deserialize(pluginQuery))
			.Where(a => a is not null)!);

		var baseType = trigger.Instance.GetType().BaseType;
		if (baseType is null || 
		    !baseType.IsGenericType ||
		    baseType.GetGenericTypeDefinition() != typeof(TriggerInstance<>) ||
		    serializedTrigger.Data is not { } data) return instance;

		DeserializeGenericTriggerMethod.MakeGenericMethod(baseType.GenericTypeArguments)
			.Invoke(null, [instance, data]);

		return instance;
	}
	
	private static readonly MethodInfo DeserializeGenericTriggerMethod =
		typeof(DeviceSerializationExtensions).GetMethod(nameof(DeserializeGenericTriggerOptions), StaticNonPublic)!;

	private static void DeserializeGenericTriggerOptions<T>(TriggerInstance<T> instance, JsonElement data)
		where T : class, new() {
		if (data.Deserialize<T>() is not { } dataObject) return;
		instance.Options = dataObject;
	}

	public static async Task<ScreenItem?> Deserialize(this SerializedScreenItem serializedItem, Input input, IPluginQuery pluginQuery) {
		var triggers = (await Task.WhenAll(
			serializedItem.Triggers.Select(sT => sT.Deserialize(pluginQuery))
			)).Where(t => t is not null).Select(t => t!);

		var instance = input is IInputDisplay
			? new RenderableScreenItem(null, triggers)
			: new ScreenlessItem(null, triggers);

		if (instance is not ScreenItem.IRenderable renderable) return instance;
		renderable.RendererName = serializedItem.RendererName;

		if (serializedItem is not { RendererSettings: { } data, RendererName: { } name }) return instance;
			
		if (pluginQuery.SpecificRenderer(name) is not { } renderer) {
			_l.Warn($"Failed to load renderer, '{name}' no longer exists");
			return instance;
		}
				
		var baseType = renderer.Instance.GetType().BaseType;
		if (baseType is null || 
		    !baseType.IsGenericType ||
		    baseType.GetGenericTypeDefinition() != typeof(Renderer<>)) return instance;

		renderable.RendererSettings = data.Deserialize(baseType.GenericTypeArguments[0]);

		return instance;
	}

	public static async Task<Screen?> Deserialize(this SerializedScreen serializedScreen, Core core,
		IPluginQuery pluginQuery) {
		var screen = core.NewScreen(serializedScreen.CanWrite);
		if (screen is not ScreenImpl screenImpl) return null; // This literally will never happen

		foreach (var ((input, item), index) in core.Inputs.Zip(serializedScreen.Items)
			         .Select((t, i) => (t, i))) {
			if (item is null) continue;
			if (await item.Deserialize(input, pluginQuery) is not { } screenItem) continue;
			screenImpl.AssignItem(index, screenItem);
		}

		return screenImpl;
	}

	public static async Task LoadConfigIntoCore(this CoreImpl core, DeviceConfig config, IPluginQuery pluginQuery) {
		var screens = (await Task.WhenAll(config.ScreenStack.Select(s => s.Deserialize(core, pluginQuery))))
			.Where(s => s is not null).Select(s => s!);
		
		core.CurrentScreenImpl?.DetachFromInputs();
		
		lock (core._screenStack) {
			core._screenStack.Clear();

			foreach (var screen in screens) {
				core._screenStack.Push(screen);
			}
		}

		core.CurrentScreenImpl?.AttachToInputs();
	}
}