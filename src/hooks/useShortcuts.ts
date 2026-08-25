import { useEffect } from 'react';
import { toast } from 'sonner';
import { useWorkspaceStore } from '../store/workspaceStore';

export function useShortcuts(setShowCommandPalette: React.Dispatch<React.SetStateAction<boolean>>) {
  const { 
    activeRequest, 
    updateRequest, 
    environments, 
    setActiveEnvironment 
  } = useWorkspaceStore();

  // Atalhos Globais
  useEffect(() => {
    const handleShortcuts = (e: KeyboardEvent) => {
      // Ctrl + P ou Ctrl + K p/ Command Palette
      if ((e.ctrlKey || e.metaKey) && (e.key === 'p' || e.key === 'k')) {
        e.preventDefault();
        setShowCommandPalette(prev => !prev);
      }
      // Ctrl + Enter p/ Send
      if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
        e.preventDefault();
        window.dispatchEvent(new CustomEvent('trigger-send'));
      }
      // Ctrl + S p/ Save
      if ((e.ctrlKey || e.metaKey) && e.key === 's') {
        e.preventDefault();
        if (activeRequest) {
          updateRequest(activeRequest);
          toast.success("Requisição salva!");
        }
      }
      // Ctrl + 1, 2, 3... p/ trocar ambiente
      if ((e.ctrlKey || e.metaKey) && !isNaN(Number(e.key)) && e.key !== '0') {
        const index = Number(e.key) - 1;
        if (environments[index]) {
          e.preventDefault();
          setActiveEnvironment(environments[index].id);
          toast.success(`Ambiente: ${environments[index].name}`);
        }
      }
    };
    window.addEventListener('keydown', handleShortcuts);
    return () => window.removeEventListener('keydown', handleShortcuts);
  }, [activeRequest, updateRequest, environments, setActiveEnvironment, setShowCommandPalette]);

  // Listener para execução via Command Palette
  useEffect(() => {
    const handleTriggerSend = () => {
       const sendBtn = document.querySelector('.send-btn') as HTMLButtonElement;
       sendBtn?.click();
    };
    window.addEventListener('trigger-send', handleTriggerSend);
    return () => window.removeEventListener('trigger-send', handleTriggerSend);
  }, []);
}
