using System.Collections.Generic;
using Streamduck.Inputs;
using Streamduck.Plugins;
using Streamduck.Triggers;

namespace Streamduck.Cores.ScreenItems;

public class TypedRenderableScreenItem<T> : ScreenlessItem, ScreenItem.IRenderable<T> where T : class, new() {
	public TypedRenderableScreenItem() { }
	internal TypedRenderableScreenItem(Input? input, IEnumerable<TriggerInstance> triggers) : base(input, triggers) { }
	
	public NamespacedName? RendererName { get; set; }
	public T? RendererSettings { get; set; }
}