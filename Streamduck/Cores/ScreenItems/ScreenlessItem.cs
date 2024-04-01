// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System.Collections.Generic;
using Streamduck.Inputs;
using Streamduck.Triggers;

namespace Streamduck.Cores.ScreenItems;

public class ScreenlessItem : ScreenItem {
	private readonly List<TriggerInstance> _triggers = [];

	public Input? AssociatedInput;
	public ScreenlessItem() { }

	internal ScreenlessItem(Input? input, IEnumerable<TriggerInstance> triggers) {
		AssociatedInput = input;
		_triggers.AddRange(triggers);
	}

	public override IReadOnlyCollection<TriggerInstance> Triggers => _triggers;

	public override void AddTrigger(TriggerInstance trigger, bool attachToInput = true) {
		_triggers.Add(trigger);

		if (attachToInput && AssociatedInput is { } input) trigger.Attach(input);
	}

	public override bool RemoveTrigger(TriggerInstance trigger) {
		if (AssociatedInput is { } input) trigger.Detach(input);
		return _triggers.Remove(trigger);
	}

	public override void Attach(Input input) {
		AssociatedInput = input;

		foreach (var trigger in _triggers) {
			trigger.Attach(input);
		}
	}

	public override void Detach() {
		if (AssociatedInput is null) return;

		foreach (var trigger in _triggers) {
			trigger.Detach(AssociatedInput);
		}

		AssociatedInput = null;
	}
}