using System.Reflection;
using Streamduck.Cores;
using Streamduck.Cores.ScreenItems;
using Streamduck.Rendering;

namespace Streamduck.Plugins.Extensions;

public static class ScreenItemExtensions {
	public static ScreenItem ReplaceRenderer(this ScreenItem item, Renderer renderer) {
		if (item is not ScreenItem.IRenderable renderable) return item;

		if (!(renderer.GetType().BaseType?.IsGenericType ?? false)
		    || renderer.GetType().BaseType!.GetGenericTypeDefinition() != typeof(Renderer<>))
			return item is ScreenlessItem screenlessItem
				? new RenderableScreenItem(screenlessItem.AssociatedInput, screenlessItem.Triggers)
				: new RenderableScreenItem(null, item.Triggers);
		
		var typeArgument = renderer.GetType().BaseType!.GetGenericArguments()[0];
		return (ScreenItem) GenericRendererMethod.MakeGenericMethod(typeArgument)
			.Invoke(null, [item, renderer])!;
	}

	private const BindingFlags StaticNonPublic = BindingFlags.Static | BindingFlags.NonPublic;
	private static readonly MethodInfo GenericRendererMethod =
		typeof(ScreenItemExtensions).GetMethod(nameof(GenericRenderer), StaticNonPublic)!;
	private static TypedRenderableScreenItem<T> GenericRenderer<T>(ScreenItem item, Renderer<T> renderer) where T : class, new() =>
		item is ScreenlessItem screenlessItem 
			? new TypedRenderableScreenItem<T>(screenlessItem.AssociatedInput, screenlessItem.Triggers)
			: new TypedRenderableScreenItem<T>(null, item.Triggers);
}