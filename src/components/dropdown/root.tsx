

import { Slot, component$, type Signal, useTask$, useSignal, useComputed$, type PropsOf, useId, useContextProvider } from "@builder.io/qwik";
import DropdownContextId, { type DropdownContext } from "./context";
import { useSelect } from "./use-select";

export type TItemsMap = Map<number, { label: string, value: string, displayValue: string, disabled: boolean }>

export type InternalSelectProps = {
  _valuePropsIndex?: number | null
  _label?: boolean
  _itemsMap: TItemsMap

}


export type DropdownProps = Omit<PropsOf<'div'>, 'onChange$'> & {
  'bind:value'?: Signal<string>,
  'bind:open'?: Signal<boolean>,
  'bind:displayText'?: Signal<string>,


  loop?: boolean

  value?: string
}


export const DropdwonImpl = component$((props: DropdownProps & InternalSelectProps) => {

  const { _itemsMap,
    loop: givenLoop,
    _valuePropsIndex: givenValuePropIndex,
    _label,
    ...rest } = props

  const itemsMapSig = useComputed$(() => {
    return _itemsMap;
  });

  const selectedIndexSetSig = useSignal<Set<number>>(
    new Set(givenValuePropIndex ? [givenValuePropIndex] : [])
  )
  // refs
  const rootRef = useSignal<HTMLDivElement>();
  const triggerRef = useSignal<HTMLButtonElement>();
  const popoverRef = useSignal<HTMLElement>();
  const listboxRef = useSignal<HTMLUListElement>();
  const labelRef = useSignal<HTMLDivElement>();
  const groupRef = useSignal<HTMLDivElement>();

  const loop = givenLoop ?? false;
  // ids
  const localId = useId();
  const listboxId = `${localId}-listbox`;
  const labelId = `${localId}-label`;
  const valueId = `${localId}-value`;

  const highlightedIndexSig = useSignal<number | null>(givenValuePropIndex ?? null);
  const currDisplayValueSig = useSignal<string | string[]>();
  const isListboxOpenSig = useSignal<boolean>(false);

  const context: DropdownContext = {
    itemsMapSig,
    currDisplayValueSig,
    triggerRef,
    popoverRef,
    listboxRef,
    labelRef,
    groupRef,
    localId,
    highlightedIndexSig,
    selectedIndexSetSig,
    isListboxOpenSig,
    // scrollOptions,
    loop,
    // multiple,
    // name,
    // required,
    // disabled,
  };

  useContextProvider(DropdownContextId, context);
  const { getActiveDescendant$, selectionManager$ } = useSelect();

  const activeDescendantSig = useComputed$(() => {
    if (isListboxOpenSig.value) {
      return getActiveDescendant$(highlightedIndexSig.value ?? -1);
    } else {
      return '';
    }
  });

  useTask$(async function updateConsumerProps({ track }) {
    const bindValueSig = props['bind:value']
    const bindDisplayTextSig = props['bind:displayText']
    track(() => selectedIndexSetSig.value)

    const values = [];
    const displayValues = [];

    for (const index of selectedIndexSetSig.value) {
      const item = itemsMapSig.value.get(index);

      if (item) {
        values.push(item.value);
        displayValues.push(item.displayValue);
      }
    }

    // sync the user's given signal when an option is selected
    if (bindValueSig && bindValueSig.value) {
      const currUserSigValues = JSON.stringify(bindValueSig.value);
      const newUserSigValues = JSON.stringify(values);

      if (currUserSigValues !== newUserSigValues) {
        bindValueSig.value = values[0];
      }

    }
    // sync the user's given signal for the display value
    if (bindDisplayTextSig) {
      bindDisplayTextSig.value = displayValues[0];
    }
  })

  useTask$(async function reactiveUserValue({ track }) {
    const bindValueSig = props['bind:value'];
    if (!bindValueSig) return;
    track(() => bindValueSig.value);

    for (const [index, item] of itemsMapSig.value) {
      if (bindValueSig.value.includes(item.value)) {
        await selectionManager$(index, 'add');

      } else {
        await selectionManager$(index, 'remove');
      }
    }
  });

  return (
    <div
      role="combobox"
      ref={rootRef}
      data-open={context.isListboxOpenSig.value ? '' : undefined}
      data-closed={!context.isListboxOpenSig.value ? '' : undefined}
      data-disabled={context.disabled ? '' : undefined}
      aria-controls={listboxId}
      aria-expanded={context.isListboxOpenSig.value}
      aria-haspopup="listbox"
      aria-activedescendant={activeDescendantSig.value}
      aria-labelledby={_label ? labelId : valueId}
      // aria-multiselectable={context.multiple ? 'true' : undefined}
      {...rest}
    >
      <Slot />
    </div>
  )
})
