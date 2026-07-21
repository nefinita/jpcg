import { useId } from "react";
import clsx from "../utils/clsx";
import styles from "./AttributeEditorPage.module.css";

interface BaseFieldProps {
  label: string;
  span?: boolean;
}

interface TextFieldProps extends BaseFieldProps {
  value: string;
  onChange: (value: string) => void;
}

export function TextField({ label, value, span = false, onChange }: TextFieldProps) {
  const id = useId();
  return (
    <label className={clsx(styles.field, span && styles.spanTwo)} htmlFor={id}>
      <span>{label}</span>
      <input id={id} type="text" value={value} onChange={(event) => onChange(event.target.value)} />
    </label>
  );
}

interface NumberFieldProps extends BaseFieldProps {
  value: number;
  min?: number;
  max?: number;
  step?: number;
  disabled?: boolean;
  onChange: (value: number) => void;
}

export function NumberField({
  label,
  value,
  min = 0,
  max,
  step = 1,
  span = false,
  disabled = false,
  onChange,
}: NumberFieldProps) {
  const id = useId();
  return (
    <label className={clsx(styles.field, span && styles.spanTwo)} htmlFor={id}>
      <span>{label}</span>
      <input
        id={id}
        type="number"
        value={value}
        min={min}
        max={max}
        step={step}
        disabled={disabled}
        onChange={(event) => {
          const nextValue = Number(event.target.value);
          onChange(Number.isFinite(nextValue) ? nextValue : 0);
        }}
      />
    </label>
  );
}

interface SelectFieldProps extends BaseFieldProps {
  value: number;
  options: ReadonlyArray<readonly [number, string]>;
  onChange: (value: number) => void;
}

export function SelectField({ label, value, options, span = false, onChange }: SelectFieldProps) {
  const id = useId();
  return (
    <label className={clsx(styles.field, span && styles.spanTwo)} htmlFor={id}>
      <span>{label}</span>
      <select id={id} value={value} onChange={(event) => onChange(Number(event.target.value))}>
        {options.map(([optionValue, optionLabel]) => (
          <option key={optionValue} value={optionValue}>
            {optionValue} · {optionLabel}
          </option>
        ))}
      </select>
    </label>
  );
}

interface ToggleFieldProps {
  label: string;
  value: boolean;
  onChange: (value: boolean) => void;
}

export function ToggleField({ label, value, onChange }: ToggleFieldProps) {
  return (
    <div className={styles.toggleRow}>
      <span className={styles.toggleLabel}>{label}</span>
      <button
        type="button"
        className={clsx(styles.switch, value && styles.switchOn)}
        role="switch"
        aria-checked={value}
        aria-label={label}
        title={label}
        onClick={() => onChange(!value)}
      >
        <span />
      </button>
    </div>
  );
}
