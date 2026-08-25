// src/components/WebSocketDebugger.tsx
import { useState, useEffect, useRef } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { Send, Power, PowerOff, Trash2 } from 'lucide-react';
import { toast } from 'sonner';

interface Message {
  text: string;
  type: 'text' | 'binary' | 'info';
  direction: 'in' | 'out';
  timestamp: string;
}

interface WebSocketDebuggerProps {
  id: string;
  url: string;
}

export function WebSocketDebugger({ id, url }: WebSocketDebuggerProps) {
  const [status, setStatus] = useState<'connected' | 'disconnected' | 'connecting'>('disconnected');
  const [messages, setMessages] = useState<Message[]>([]);
  const [input, setInput] = useState("");
  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const unlistenMsg = listen<any>('ws-message', (event) => {
      if (event.payload.connectionId === id) {
        setMessages(prev => [...prev.slice(-99), {
          text: event.payload.message,
          type: event.payload.type,
          direction: 'in',
          timestamp: new Date().toLocaleTimeString()
        }]);
      }
    });

    const unlistenStatus = listen<any>('ws-status', (event) => {
      if (event.payload.connectionId === id) {
        setStatus(event.payload.status);
      }
    });

    return () => {
      unlistenMsg.then(u => u());
      unlistenStatus.then(u => u());
    };
  }, [id]);

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [messages]);

  const handleConnect = async () => {
    setStatus('connecting');
    try {
      await invoke('ws_connect', { id, url });
    } catch (e: any) {
      toast.error("WebSocket connection failed", { description: e });
      setStatus('disconnected');
    }
  };

  const handleDisconnect = async () => {
    try {
      await invoke('ws_disconnect', { id });
    } catch (e) {}
  };

  const handleSend = async () => {
    if (!input || status !== 'connected') return;
    try {
      await invoke('ws_send', { id, message: input });
      setMessages(prev => [...prev.slice(-99), {
        text: input,
        type: 'text',
        direction: 'out',
        timestamp: new Date().toLocaleTimeString()
      }]);
      setInput("");
    } catch (e) {
      toast.error("Failed to send message");
    }
  };

  return (
    <div className="ws-debugger">
      <div className="ws-toolbar">
        <div className={`ws-status-indicator ${status}`}>
          {status === 'connected' ? 'Connected' : status === 'connecting' ? 'Connecting...' : 'Disconnected'}
        </div>
        <div className="ws-actions">
          {status === 'disconnected' ? (
            <button onClick={handleConnect} className="ws-btn connect">
              <Power size={14} /> Connect
            </button>
          ) : (
            <button onClick={handleDisconnect} className="ws-btn disconnect">
              <PowerOff size={14} /> Disconnect
            </button>
          )}
          <button onClick={() => setMessages([])} className="ws-btn clear" title="Clear Messages">
            <Trash2 size={14} />
          </button>
        </div>
      </div>

      <div className="ws-messages" ref={scrollRef}>
        {messages.length === 0 && (
          <div className="ws-empty">No messages yet. Connect and start debugging.</div>
        )}
        {messages.map((msg, i) => (
          <div key={i} className={`ws-msg-row ${msg.direction}`}>
            <span className="ws-msg-time">{msg.timestamp}</span>
            <span className="ws-msg-dir">{msg.direction === 'in' ? '▼' : '▲'}</span>
            <div className="ws-msg-content">{msg.text}</div>
          </div>
        ))}
      </div>

      <div className="ws-input-area">
        <input 
          type="text" 
          value={input} 
          onChange={(e) => setInput(e.target.value)}
          placeholder="Type message to send..."
          onKeyDown={(e) => e.key === 'Enter' && handleSend()}
          disabled={status !== 'connected'}
        />
        <button onClick={handleSend} disabled={status !== 'connected' || !input}>
          <Send size={16} />
        </button>
      </div>
    </div>
  );
}
