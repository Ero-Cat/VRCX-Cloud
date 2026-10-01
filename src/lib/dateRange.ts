export function toIsoRangeStart(value: unknown) {
    if (!value) {
        return '';
    }

    const date = new Date(`${value}T00:00:00`);
    return Number.isNaN(date.valueOf()) ? '' : date.toISOString();
}

export function toIsoRangeEnd(value: unknown) {
    if (!value) {
        return '';
    }

    const date = new Date(`${value}T23:59:59.999`);
    return Number.isNaN(date.valueOf()) ? '' : date.toISOString();
}

export function parseDateInput(value: unknown) {
    const normalizedValue = String(value ?? '').trim();
    if (!/^\d{4}-\d{2}-\d{2}$/.test(normalizedValue)) {
        return undefined;
    }
    const [year, month, day] = normalizedValue
        .split('-')
        .map((part) => Number.parseInt(part, 10));
    const date = new Date(year, month - 1, day);
    return Number.isNaN(date.valueOf()) ? undefined : date;
}

export function toDateInputValue(date: unknown) {
    if (!(date instanceof Date) || Number.isNaN(date.valueOf())) {
        return '';
    }
    const year = date.getFullYear();
    const month = String(date.getMonth() + 1).padStart(2, '0');
    const day = String(date.getDate()).padStart(2, '0');
    return `${year}-${month}-${day}`;
}
