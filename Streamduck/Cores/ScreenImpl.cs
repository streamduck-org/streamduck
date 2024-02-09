using System.Collections.Generic;
using System.Linq;
using System.Reflection;
using Streamduck.Cores.ScreenItems;
using Streamduck.Inputs;
using Streamduck.Plugins;
using Streamduck.Plugins.Extensions;
using Streamduck.Rendering;

namespace Streamduck.Cores;

public class ScreenImpl(IEnumerable<Input> inputs, IPluginQuery pluginQuery) : Screen {
	private readonly Input[] _inputs = inputs.ToArray();
	private readonly ScreenItem?[] _items = [];
	private readonly IPluginQuery _pluginQuery = pluginQuery;

	public override IReadOnlyCollection<ScreenItem?> Items => _items;

	public override ScreenItem CreateItem(int index) => throw new System.NotImplementedException();
	
	private const BindingFlags StaticNonPublic = BindingFlags.Static | BindingFlags.NonPublic;
	private static readonly MethodInfo GenericRendererMethod =
		typeof(ScreenImpl).GetMethod(nameof(GenericRenderer), StaticNonPublic)!;
	private static TypedRenderableScreenItem<T> GenericRenderer<T>(Renderer<T> renderer) where T : class, new() => new();

	public override ScreenItem DeleteItem(int index) => throw new System.NotImplementedException();
}