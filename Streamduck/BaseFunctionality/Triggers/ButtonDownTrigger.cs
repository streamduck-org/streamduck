// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System;
using System.Threading.Tasks;
using Streamduck.Attributes;
using Streamduck.Data;
using Streamduck.Inputs;
using Streamduck.Triggers;

namespace Streamduck.BaseFunctionality.Triggers;

[AutoAdd]
public class ButtonDownTrigger : Trigger {
	public override string Name => "Button Down Trigger";
	public override string? Description => "Triggers action when button is pressed down";

	public override bool IsApplicableTo(Input input) => input is IInputButton or IInputTouchScreen or IInputToggle;

	public override Task<TriggerInstance> CreateInstance() =>
		Task.FromResult((TriggerInstance)new ButtonDownTriggerInstance(this));
}

public class ButtonDownTriggerInstance(Trigger original) : TriggerInstance(original) {
	public override void Attach(Input input) {
		switch (input) {
			case IInputButton button:
				button.ButtonPressed += InvokeActions;
				break;
			case IInputToggle toggle:
				toggle.ToggleStateChanged += Toggle;
				break;
			case IInputTouchScreen touchScreen:
				touchScreen.TouchScreenPressed += TouchScreen;
				break;
		}
	}

	private void TouchScreen(Int2 _) {
		InvokeActions();
	}

	private void Toggle(bool down) {
		if (down) InvokeActions();
	}

	public override void Detach(Input input) {
		switch (input) {
			case IInputButton button:
				button.ButtonPressed -= InvokeActions;
				break;
			case IInputToggle toggle:
				toggle.ToggleStateChanged -= Toggle;
				break;
			case IInputTouchScreen touchScreen:
				touchScreen.TouchScreenPressed -= TouchScreen;
				break;
		}
	}
}