using System;
using System.Collections.Generic;
using Streamduck.Inputs;
using Streamduck.Triggers;

namespace Streamduck.Cores.ScreenItems;

public class ScreenlessItem : ScreenItem {
	public ScreenlessItem() { }
	
	internal ScreenlessItem(Input? input, IEnumerable<TriggerInstance> triggers) {
		AssociatedInput = input;
		_triggers.AddRange(triggers);
	}
	
	public Input? AssociatedInput;
	
	private readonly List<TriggerInstance> _triggers = [];

	public override IEnumerable<TriggerInstance> Triggers => _triggers;
	
	public override void AddTrigger(TriggerInstance trigger, bool attachToInput = true) {
		_triggers.Add(trigger);

		if (attachToInput && AssociatedInput is { } input) {
			trigger.Attach(input);
		}
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

	public override void Detach(Input input) {
		foreach (var trigger in _triggers) {
			trigger.Detach(input);
		}
		
		AssociatedInput = null;
	}
}