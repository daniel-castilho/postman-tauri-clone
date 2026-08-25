import { Folder, FileJson, PlusCircle, FolderPlus, Trash2, Clock, Copy, Download, Share2, Play, FileUp, BookOpen, Code, Terminal, Activity } from 'lucide-react';
import React, { useState } from 'react';
import { motion } from 'framer-motion';
import { invoke } from "@tauri-apps/api/core";
import { open as openFile } from "@tauri-apps/plugin-dialog";
import { toast } from "sonner";
import { useWorkspaceStore, Collection, CollectionItem } from '../store/workspaceStore';
import { MockServer } from './MockServer';
import { MonitorsPanel } from './MonitorsPanel';
import './Sidebar.css';

export function Sidebar() {
  const { 
    collections, history, isLoading, workspacePath, 
    activeRequest, setActiveRequest, addCollection, 
    addRequestToCollection, addFolderToCollection, 
    deleteRequest, clearHistory, duplicateRequest,
    importCollection, reorderItems, exportWorkspace, loadCollections,
    designs, activeDesignId, setActiveDesign, loadDesigns, createDesign, deleteDesign 
  } = useWorkspaceStore();

  const [sidebarTab, setSidebarTab] = useState<'Collections' | 'History' | 'Mock' | 'Monitor' | 'Designs'>('Collections');
  const [draggedIdx, setDraggedIdx] = useState<number | null>(null);
  const [dropTargetIdx, setDropTargetIdx] = useState<number | null>(null);
  const [dropTargetColId, setDropTargetColId] = useState<string | null>(null);
  
  const [showDocsModal, setShowDocsModal] = useState(false);
  const [generatedDocs, setGeneratedDocs] = useState("");

  const handleAddCollection = () => {
    const name = prompt("Nome da Nova Coleção:");
    if (name) addCollection(name);
  };

  const handleAddRequest = (e: React.MouseEvent, colId: string) => {
    e.stopPropagation();
    addRequestToCollection(colId);
  };

  const handleAddFolder = (e: React.MouseEvent, colId: string) => {
    e.stopPropagation();
    const name = prompt("Nome da Nova Pasta:");
    if (name) addFolderToCollection(colId, name);
  };

  const handleDeleteRequest = (e: React.MouseEvent, reqId: string) => {
    e.stopPropagation();
    if (confirm("Tem certeza que deseja deletar esta requisição?")) {
      deleteRequest(reqId);
    }
  };

  const handleImportOpenApi = async () => {
    try {
      const file = await openFile({
        multiple: false,
        filters: [{ name: 'OpenAPI', extensions: ['json', 'yaml', 'yml'] }]
      });
      if (file && !Array.isArray(file)) {
        const content = await invoke<string>("read_file_text", { path: file as string });
        await invoke("import_openapi", { content, workspacePath });
        toast.success("Coleção importada com sucesso!");
        await loadCollections();
      }
    } catch (e: any) {
      toast.error("Erro ao importar", { description: e.toString() });
    }
  };

  const handleGenerateDocs = async (collection: Collection) => {
    try {
      const docs = await invoke<string>("generate_docs", { collection });
      setGeneratedDocs(docs);
      setShowDocsModal(true);
    } catch (e: any) {
      toast.error("Erro ao gerar documentação", { description: e.toString() });
    }
  };

  const handleDuplicateRequest = (e: React.MouseEvent, reqId: string) => {
    e.stopPropagation();
    duplicateRequest(reqId);
  };

  const handleDragStart = (idx: number) => {
    setDraggedIdx(idx);
  };

  const handleDragOver = (e: React.DragEvent, colId: string, idx: number) => {
    e.preventDefault();
    setDropTargetColId(colId);
    setDropTargetIdx(idx);
  };

  const handleDragEnd = () => {
    setDraggedIdx(null);
    setDropTargetIdx(null);
    setDropTargetColId(null);
  };

  const handleDrop = async (e: React.DragEvent, colId: string, targetIdx: number) => {
    e.preventDefault();
    if (draggedIdx === null || draggedIdx === targetIdx) return;

    const collection = collections.find(c => c.id === colId);
    if (!collection) return;

    const newItems = [...collection.items];
    const itemToMove = newItems[draggedIdx];
    newItems.splice(draggedIdx, 1);
    newItems.splice(targetIdx, 0, itemToMove);

    await reorderItems(colId, newItems);
    handleDragEnd();
  };

  const renderItem = (item: CollectionItem, idx: number, colId: string) => {
    if ('Folder' in item) {
      return (
        <div key={idx} className="sidebar-folder">
          <div className="sidebar-item-header">
            <div className="sidebar-item-info">
              <Folder size={16} />
              <span>{item.Folder.name}</span>
            </div>
            <div className="sidebar-item-actions">
              <button 
                className="run-btn-sidebar" 
                title="Run Collection"
                onClick={() => {
                   window.dispatchEvent(new CustomEvent('open-runner', { detail: item.Folder.items }));
                }}
              >
                <Play size={12} fill="currentColor" />
              </button>
            </div>
          </div>
          <div className="sidebar-folder-items">
            {item.Folder.items.map((subItem: any, subIdx: number) => renderItem(subItem, subIdx, colId))}
          </div>
        </div>
      );
    } else {
      const isDragging = draggedIdx === idx;
      const isDropTarget = dropTargetIdx === idx && dropTargetColId === colId;

      return (
        <motion.div 
          key={item.Request.id}
          layout
          initial={{ opacity: 0, x: -10 }}
          animate={{ opacity: 1, x: 0 }}
          exit={{ opacity: 0, x: 10 }}
          className={`sidebar-request ${activeRequest?.id === item.Request.id ? 'active' : ''} ${isDragging ? 'dragging' : ''} ${isDropTarget ? 'drop-target' : ''}`}
          onClick={() => setActiveRequest(item.Request)}
          draggable
          onDragStart={() => handleDragStart(idx)}
          onDragOver={(e: React.DragEvent) => handleDragOver(e, colId, idx)}
          onDragEnd={handleDragEnd}
          onDrop={(e: React.DragEvent) => handleDrop(e, colId, idx)}
        >
          <div className="sidebar-request-info">
            <FileJson size={14} />
            <span className="method-badge" data-method={typeof item.Request.method === 'string' ? item.Request.method : 'CUSTOM'}>
              {typeof item.Request.method === 'string' ? item.Request.method : `CUSTOM (${item.Request.method.CUSTOM})`}
            </span>
            <span className="request-url" title={item.Request.url}>{item.Request.url}</span>
          </div>
          <div className="sidebar-item-actions">
            <button 
              className="delete-req-btn" 
              title="Deletar" 
              onClick={(e) => handleDeleteRequest(e, item.Request.id)}
            >
              <Trash2 size={14} />
            </button>
            <button 
              className="delete-req-btn" 
              title="Duplicar" 
              onClick={(e) => handleDuplicateRequest(e, item.Request.id)}
              style={{ marginRight: '4px' }}
            >
              <Copy size={13} />
            </button>
          </div>
        </motion.div>
      );
    }
  };

  return (
    <aside className="sidebar">
      <div className="sidebar-tabs">
        <button 
          className={`sidebar-tab-btn ${sidebarTab === 'Collections' ? 'active' : ''}`}
          onClick={() => setSidebarTab('Collections')}
        >Collections</button>
        <button 
          className={`sidebar-tab-btn ${sidebarTab === 'History' ? 'active' : ''}`}
          onClick={() => setSidebarTab('History')}
        >History</button>
        <button 
          className={`sidebar-tab-btn ${sidebarTab === 'Mock' ? 'active' : ''}`}
          onClick={() => setSidebarTab('Mock')}
        >Mock</button>
        <button 
          className={`sidebar-tab-btn ${sidebarTab === 'Monitor' ? 'active' : ''}`}
          onClick={() => setSidebarTab('Monitor')}
        >Monitor</button>
        <button 
          className={`sidebar-tab-btn ${sidebarTab === 'Designs' ? 'active' : ''}`}
          onClick={() => {
            setSidebarTab('Designs');
            loadDesigns();
          }}
        >Designs</button>
      </div>

      {sidebarTab === 'Collections' ? (
        <>
          <div className="sidebar-header">
            <h2>Coleções</h2>
            <button className="add-btn" title="Nova Coleção" onClick={handleAddCollection}>
              <PlusCircle size={18} />
            </button>
            <button className="add-btn" title="Importar Coleção" onClick={importCollection} style={{ marginLeft: '8px' }}>
              <Download size={18} />
            </button>
            <button className="add-btn" title="Exportar Workspace" onClick={exportWorkspace} style={{ marginLeft: '8px' }}>
              <Share2 size={18} />
            </button>
            <button className="add-btn" title="Importar OpenAPI/Swagger" onClick={handleImportOpenApi} style={{ marginLeft: '8px' }}>
              <FileUp size={18} />
            </button>
          </div>

          <div className="sidebar-content">
            {isLoading && <div className="loading-state">Carregando...</div>}
            {!isLoading && collections.length === 0 && (
              <div className="empty-state">
                Nenhuma coleção encontrada em {workspacePath}
              </div>
            )}
            {!isLoading && collections.map((col: Collection) => (
              <div key={col.id} className="collection-root">
                <div className="collection-header">
                  <div className="collection-header-title">
                    <Folder size={16} className="collection-icon" />
                    <strong>{col.name}</strong>
                  </div>
                  <div className="collection-actions">
                    <button 
                      className="add-req-btn" 
                      title="Nova Pasta" 
                      onClick={(e) => handleAddFolder(e, col.id)}
                    >
                      <FolderPlus size={14} />
                    </button>
                    <button 
                      className="add-req-btn" 
                      title="Nova Requisição" 
                      onClick={(e) => handleAddRequest(e, col.id)}
                    >
                      <PlusCircle size={14} />
                    </button>
                    <button 
                      className="add-req-btn" 
                      title="Gerar Documentação" 
                      onClick={(e) => { e.stopPropagation(); handleGenerateDocs(col); }}
                    >
                      <BookOpen size={14} />
                    </button>
                  </div>
                </div>
                <div className="collection-items">
                  {col.items.map((item, idx) => renderItem(item, idx, col.id))}
                </div>
              </div>
            ))}
          </div>
        </>
      ) : sidebarTab === 'History' ? (
        <>
          <div className="sidebar-header">
            <h2>History</h2>
            <button className="add-btn" title="Limpar Histórico" onClick={clearHistory}>
              <Trash2 size={16} />
            </button>
          </div>
          <div className="sidebar-content">
            {history.length === 0 && <div className="empty-state">No recent requests</div>}
            {history.map((req: any, idx: number) => (
              <motion.div 
                key={req.id + idx}
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                className={`sidebar-request ${activeRequest?.id === req.id ? 'active' : ''}`}
                onClick={() => setActiveRequest(req)}
              >
                <div className="sidebar-request-info">
                  <Clock size={14} />
                  <span className="method-badge" data-method={typeof req.method === 'string' ? req.method : 'CUSTOM'}>
                    {typeof req.method === 'string' ? req.method : `CUSTOM (${req.method.CUSTOM})`}
                  </span>
                  <span className="request-url" title={req.url}>{req.url}</span>
                </div>
              </motion.div>
            ))}
          </div>
        </>
      ) : sidebarTab === 'Mock' ? (
        <MockServer />
      ) : sidebarTab === 'Designs' ? (
        <>
          <div className="sidebar-header">
            <h2>Designs & Specs</h2>
            <button className="add-btn" title="Novo Design" onClick={() => {
              const name = prompt("Nome do Design (ex: Store API):");
              if (name) createDesign(name, "yaml");
            }}>
              <PlusCircle size={18} />
            </button>
          </div>
          <div className="sidebar-content">
            {designs.length === 0 && <div className="empty-state">No designs found. Create one to begin.</div>}
            {designs.map((design: any) => (
              <motion.div 
                key={design.id}
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                className={`sidebar-request design-item ${activeDesignId === design.id ? 'active' : ''}`}
                onClick={() => setActiveDesign(design.id)}
              >
                <div className="sidebar-request-info">
                  <Code size={14} />
                  <span className="request-url">{design.name}</span>
                  <span className="design-format-tag">{design.format}</span>
                </div>
              </motion.div>
            ))}
          </div>
        </>
      ) : (
        <MonitorsPanel />
      )}

      {showDocsModal && (
        <div className="cookies-overlay" onClick={() => setShowDocsModal(false)}>
          <div className="cookies-modal code-snippet-modal" onClick={e => e.stopPropagation()}>
            <div className="cookies-modal-header">
              <h3>Documentação da Coleção</h3>
              <button className="close-cookies-btn" onClick={() => setShowDocsModal(false)}>×</button>
            </div>
            <div className="cookies-modal-body">
              <div className="curl-container">
                <pre className="curl-code">{generatedDocs}</pre>
                <button className="copy-curl-btn" onClick={() => {
                  navigator.clipboard.writeText(generatedDocs);
                  toast.success("Documentação copiada!");
                }}>
                  <Copy size={14} />
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </aside>
  );
}
