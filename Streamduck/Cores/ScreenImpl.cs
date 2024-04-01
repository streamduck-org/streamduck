// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System;
using System.Collections.Generic;
using System.Linq;
using Streamduck.Cores.ScreenItems;
using Streamduck.Inputs;
using Streamduck.Plugins;

namespace Streamduck.Cores;

public class ScreenImpl(Core core, IReadOnlyCollection<Input> inputs, IPluginQuery pluginQuery) : Screen {
	private readonly Input[] _inputs = inputs.ToArray();
	private readonly ScreenItem?[] _items = new ScreenItem?[inputs.Count];

	public override Core AssociatedCore => core;
	public override IReadOnlyCollection<ScreenItem?> Items => _items;

	public override ScreenItem CreateItem(int index) {
		ArgumentOutOfRangeException.ThrowIfGreaterThanOrEqual(index, _inputs.Length);
		if (_items[index] != null) throw new ArgumentException("Item already exists on the index", nameof(index));

		var input = _inputs[index];
		var item = CreateFromInput(input);

		_items[index] = item;

		if (core.CurrentScreen == this) item.Attach(input);

		return item;
	}

	private ScreenItem CreateFromInput(Input input) {
		if (input is not IInputDisplay display) return new ScreenlessItem();

		var renderer = pluginQuery.DefaultRenderer();

		if (renderer is null) return new ScreenlessItem();

		return new RenderableScreenItem {
			RendererName = renderer.NamespacedName,
			RendererSettings = renderer.Instance.DefaultRendererConfig
		};
	}

	public override ScreenItem DeleteItem(int index) {
		ArgumentOutOfRangeException.ThrowIfGreaterThanOrEqual(index, _inputs.Length);
		if (_items[index] == null) throw new ArgumentException("Item doesn't exist on the index", nameof(index));

		var item = _items[index]!;

		if (core.CurrentScreen == this) item.Detach();

		_items[index] = null;
		return item;
	}

	public override void AttachToInputs() {
		foreach (var (item, input) in _items.Zip(_inputs)) {
			item?.Attach(input);
		}
	}

	public override void DetachFromInputs() {
		foreach (var item in _items) {
			item?.Detach();
		}
	}
}