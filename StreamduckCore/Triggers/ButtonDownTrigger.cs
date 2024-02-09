using System;
using System.Threading.Tasks;
using Streamduck.Inputs;
using Streamduck.Triggers;

namespace StreamduckCore.Triggers;

public class ButtonDownTrigger : Trigger {
	public override string Name => "Button Down Trigger";
	public override string? Description => "Triggers action when button is pressed down";
	public override bool IsApplicableTo(Input input) => input is IInputButton;
	public override Task<TriggerInstance> CreateInstance() => 
		Task.FromResult((TriggerInstance) new ButtonDownTriggerInstance(this));
}

public class ButtonDownTriggerInstance(Trigger original) : TriggerInstance(original) {
	public override event Action? Triggered;
	
	public override void Attach(Input input) {
		(input as IInputButton)!.ButtonPressed += OnButtonPressed;
	}
	
	public override void Detach(Input input) {
		(input as IInputButton)!.ButtonPressed -= OnButtonPressed;
	}

	private void OnButtonPressed() {
		Triggered?.Invoke();
	}
}