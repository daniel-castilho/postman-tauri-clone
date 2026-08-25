import docx

doc = docx.Document()

# Load text
with open('conversation.txt', 'r', encoding='utf-8') as f:
    lines = f.readlines()

for line in lines:
    doc.add_paragraph(line.strip())

doc.save('conversa.docx')
