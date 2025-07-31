use std::{
    collections::{hash_map, HashMap},
    path::Path,
};

use pathdiff::diff_paths;
use tower_lsp::lsp_types::{
    CodeAction, CodeActionOrCommand, CodeActionParams, CreateFile, CreateFileOptions,
    DocumentChangeOperation, DocumentChanges, OneOf, OptionalVersionedTextDocumentIdentifier,
    Position, Range, ResourceOp, TextDocumentEdit, TextEdit, Url, WorkspaceEdit,
};

use crate::{
    config::Settings,
    daily::filename_is_formatted,
    diagnostics::path_unresolved_references,
    link::Link,
    unique_notes::find_unique_file_path,
    vault::{Reference, Vault},
};

pub fn code_actions(
    vault: &Vault,
    params: &CodeActionParams,
    path: &Path,
    settings: &Settings,
) -> Option<Vec<CodeActionOrCommand>> {
    // Diagnostics
    // get all links for changed file

    let unresolved = path_unresolved_references(vault, path)?;

    let unresolved_file_links = unresolved;

    let code_action_unresolved = unresolved_file_links.into_iter().filter(|(_, reference)| {
        reference.data().range.start.line <= params.range.start.line
            && reference.data().range.end.line >= params.range.end.line
            && reference.data().range.start.character <= params.range.start.character
            && reference.data().range.end.character >= params.range.end.character
    });

    Some(
        code_action_unresolved
            .flat_map(|(path, reference)| {
                match reference {
                    Reference::WikiFileLink(_data) => {
                        vec!({
                          let filename = &reference.data().reference_text;

                          let mut new_path_buf = vault.root_dir().clone();
                          if filename_is_formatted(settings, filename) {
                              new_path_buf.push(&settings.daily_notes_folder);
                          } else {
                              new_path_buf.push(&settings.new_file_folder_path);
                          }
                          new_path_buf.push(filename);
                          new_path_buf.set_extension("md");

                          Url::from_file_path(&new_path_buf).ok()
                            .and_then(|new_path| {
                              Some(CodeActionOrCommand::CodeAction(CodeAction {
                                  title: format!(
                                      "Create File: {:?}",
                                      diff_paths(new_path_buf, vault.root_dir())?
                                  ),
                                  edit: Some(WorkspaceEdit {
                                      document_changes: Some(DocumentChanges::Operations(vec![
                                          DocumentChangeOperation::Op(ResourceOp::Create(CreateFile {
                                              uri: new_path,
                                              options: None,
                                              annotation_id: None,
                                          })),
                                      ])),
                                      ..Default::default()
                                  }),
                                  ..Default::default()
                              }))
                            })
                        },{
                          let note_title = &reference.data().reference_text;

                          let mut new_path_buf = vault.root_dir().clone();
                          if filename_is_formatted(settings, note_title) {
                              None
                          } else {
                              new_path_buf.push(&settings.unique_notes_folder);

                            new_path_buf = find_unique_file_path(&new_path_buf, &settings.unique_notes_format);

                            Url::from_file_path(&new_path_buf).ok()
                              .and_then(|new_path| {
                                let link = Link {
                                  refname: new_path_buf
                                      .with_extension("")
                                      .file_name()?.to_str()?.to_string(),
                                  display: Some(note_title.to_string()),
                                };

                                Some(CodeActionOrCommand::CodeAction(CodeAction {
                                    title: format!(
                                        "Create Unique Note: {:?}",
                                        diff_paths(new_path_buf, vault.root_dir())?
                                    ),
                                    edit: Some(WorkspaceEdit {
                                        document_changes: Some(DocumentChanges::Operations(vec![
                                            DocumentChangeOperation::Op(ResourceOp::Create(CreateFile {
                                                uri: new_path.clone(),
                                                options: None,
                                                annotation_id: None,
                                            })),
                                            DocumentChangeOperation::Edit(TextDocumentEdit {
                                              text_document: OptionalVersionedTextDocumentIdentifier {
                                                uri: new_path,
                                                version: None,
                                              },
                                              edits: vec![
                                                OneOf::Left(TextEdit {
                                                  range: Range {
                                                    start: Position { line: 0, character: 0 },
                                                    end: Position { line: 0, character: (note_title.len() as u32) + 2 }
                                                  },
                                                  new_text: format!("# {}", note_title.to_string()),
                                                })
                                              ]
                                            }),
                                            DocumentChangeOperation::Edit(TextDocumentEdit {
                                              text_document: OptionalVersionedTextDocumentIdentifier {
                                                uri: Url::from_file_path(path).ok()?,
                                                version: None,
                                              },
                                              edits: vec![
                                                OneOf::Left(TextEdit {
                                                  range: reference.range.0,
                                                  new_text: link.as_link_text(settings)
                                                })
                                              ]
                                            }),
                                        ])),
                                        ..Default::default()
                                    }),
                                    ..Default::default()
                                }))
                              })
                          }
                        }
                        )
                    }
                    Reference::WikiHeadingLink(_data, link_path, heading) => {


                                              vec!({
                        let mut new_path_buf = vault.root_dir().clone();
                        if filename_is_formatted(settings, link_path) {
                            new_path_buf.push(&settings.daily_notes_folder);
                        } else {
                            new_path_buf.push(&settings.new_file_folder_path);
                        }
                        new_path_buf.push(link_path);
                        new_path_buf.set_extension("md");

                        Url::from_file_path(&new_path_buf).ok()
                            .and_then(|new_path| {

                        let file = vault.ropes.get(&new_path_buf);

                        let length = match file {
                            Some(file) => file.lines().len(),
                            None => 0
                        };


                        let new_text = match file {
                            Some(..) => format!("\n\n# {}", heading),
                            None => format!("# {}", heading)
                        }; // move this calculation to the vault somehow

                          Some(CodeActionOrCommand::CodeAction(CodeAction {
                              title: format!(
                                  "Append Heading \"{}\" to file {}.md, creating it if it doesn't exist",
                                  heading,
                                  link_path
                              ),
                              edit: Some(WorkspaceEdit{
                                  document_changes: Some(DocumentChanges::Operations(vec![
                                      DocumentChangeOperation::Op(ResourceOp::Create(CreateFile {
                                          uri: new_path.clone(),
                                          annotation_id: None,
                                          options: Some(CreateFileOptions {
                                              ignore_if_exists: Some(true),
                                              overwrite: Some(false)
                                          })
                                      })),
                                      DocumentChangeOperation::Edit(TextDocumentEdit{
                                          text_document: OptionalVersionedTextDocumentIdentifier{
                                              uri: new_path,
                                              version: None
                                          },
                                          edits: vec![
                                              OneOf::Left(TextEdit{
                                                  new_text,
                                                  range: Range {
                                                      start: Position {
                                                          line: (length + 1) as u32,
                                                          character: 0
                                                      },
                                                      end: Position {
                                                          line: length as u32,
                                                          character: 0
                                                      }
                                                  }
                                              })
                                          ]
                                      })
                                  ])),
                                  ..Default::default()
                              }),
                              ..Default::default()
                          }))

                            })
                                              }
                        )
                    }
                    _ => vec!(None)
                }

            })
            .flatten()
            .collect(),
    )
}
