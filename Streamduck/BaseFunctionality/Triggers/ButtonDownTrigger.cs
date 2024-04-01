// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System.Threading.Tasks;
using Streamduck.Attributes;
using Streamduck.Inputs;
using Streamduck.Triggers;

namespace Streamduck.BaseFunctionality.Triggers;

[AutoAdd]
public class ButtonDownTrigger : Trigger {
	public override string Name => "Button Down Trigger";
	public override string? Description => "Triggers action when button is pressed down";
	public override bool IsApplicableTo(Input input) => input is IInputButton;

	public override Task<TriggerInstance> CreateInstance() =>
		Task.FromResult((TriggerInstance)new ButtonDownTriggerInstance(this));
}

public class ButtonDownTriggerInstance(Trigger original) : TriggerInstance(original) {
	public override void Attach(Input input) {
		(input as IInputButton)!.ButtonPressed += InvokeActions;
	}

	public override void Detach(Input input) {
		(input as IInputButton)!.ButtonPressed -= InvokeActions;
	}
}