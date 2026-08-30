import { useCallback, useEffect, useRef, useState } from "react";

/**
 * Faz polling a uma função assíncrona da API a cada `intervalMs`.
 * Devolve { data, error, loading, offline, refresh }.
 * `offline` fica true quando o backend dc-os-core não responde,
 * para a UI poder cair num estado de demonstração em vez de rebentar.
 */
export function usePolledApi(fn, { intervalMs = 0, deps = [] } = {}) {
  const [data, setData] = useState(null);
  const [error, setError] = useState(null);
  const [loading, setLoading] = useState(true);
  const fnRef = useRef(fn);
  fnRef.current = fn;

  const refresh = useCallback(async () => {
    try {
      const result = await fnRef.current();
      setData(result);
      setError(null);
    } catch (err) {
      setError(err);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    let cancelled = false;
    let timer;

    const run = async () => {
      if (cancelled) return;
      await refresh();
      if (!cancelled && intervalMs > 0) {
        timer = setTimeout(run, intervalMs);
      }
    };
    run();

    return () => {
      cancelled = true;
      if (timer) clearTimeout(timer);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [refresh, intervalMs, ...deps]);

  return { data, error, loading, offline: Boolean(error), refresh };
}
