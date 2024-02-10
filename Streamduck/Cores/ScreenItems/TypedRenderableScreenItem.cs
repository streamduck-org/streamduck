// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

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