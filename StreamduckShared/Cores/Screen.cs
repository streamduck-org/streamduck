using System.Collections.Generic;

namespace Streamduck.Cores; 

/**
 * Screen that can contain screen items
 */
public abstract class Screen {
	public abstract Core AssociatedCore { get; }
	public bool CanWrite { get; init; } = true;
	public abstract IReadOnlyCollection<ScreenItem?> Items { get; }

	public abstract ScreenItem CreateItem(int index);
	public abstract ScreenItem DeleteItem(int index);
	public abstract void AttachToInputs();
	public abstract void DetachFromInputs();
}