using System.Collections.Generic;
using Streamduck.Inputs;
using Streamduck.Plugins;
using Streamduck.Triggers;

namespace Streamduck.Cores.ScreenItems;

public class RenderableScreenItem : ScreenlessItem, ScreenItem.IRenderable {
	public RenderableScreenItem() { }
	internal RenderableScreenItem(Input? input, IEnumerable<TriggerInstance> triggers) : base(input, triggers) { }
	
	public NamespacedName? RendererName { get; set; }
	public object? RendererSettings { get; set; }
}