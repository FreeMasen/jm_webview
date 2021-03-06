import * as React from 'react';
import { Image, Meta, Project } from '../appState';
import StringHandler from '../services/stringHandler';
import InputGroup from './inputGroup';
import ListBox from './listBox';

interface IProjectEditorProps {
    project: Project;
    addImageHandler: () => void;
    saveHandler: (p: Project) => void;
    cancelHandler: () => void;
    moveImage: (oldPos: number, newPos: number) => void;
    deleteProject: () => void;
}

interface IProjectEditorState {
    title: string;
    subtitle: string;
    teammates: string[];
    description: string;
    images: Image[];
    selectedImage?: Image;
    newContributor: string;
    bwTitleImage: boolean;
}

export default class ProjectEditor extends React.Component<IProjectEditorProps, IProjectEditorState> {
    constructor(props) {
        super(props);
        let { meta, images, description } = props.project;
        this.state = {
            title: meta.title || '',
            subtitle: meta.subtitle || '',
            teammates: meta.teammates || [],
            images,
            description,
            newContributor: '',
            bwTitleImage: props.project.bwTitleImage || false,
        }
    }
    componentWillReceiveProps(props: IProjectEditorProps) {
        if (props.project.images != this.state.images) {
            this.setState({images: props.project.images});
        }
    }
    pageSaved() {
        console.log("pageSaved", this.state);
        let p = new Project(
            this.props.project.id,
            this.props.project.path,
            new Meta(this.state.title, this.state.subtitle, this.state.teammates),
            this.state.images,
            this.state.description,
            this.state.bwTitleImage,
        );
        this.props.saveHandler(p);
    }

    removeImages() {
        this.setState((prev, props) => {
            return {
                images: prev.images.filter((i) => i != this.state.selectedImage)
            }
        })
    }

    moveImage(up: boolean) {
        let oldPos = this.state.selectedImage ? this.state.selectedImage.position : -1;
        if (oldPos < 0 || oldPos >= this.state.images.length) return console.log('old position for this image is below 0');
        let newPos;
        if (up) {
            newPos = oldPos - 1;
        } else {
            newPos = oldPos + 1;
        }
        if (newPos < 0 || newPos >= this.state.images.length)
            return console.error('Unable to move image outside of array')
        let images = this.state.images;
        let mover = images[oldPos];
        images[oldPos] = images[newPos];
        images[newPos] = mover;
        images.forEach((e, i) => e.position = i);
        this.setState({images});
    }

    addContributor() {
        this.setState((prev, props) => {
            prev.teammates.push(prev.newContributor);
            return {
                teammates: prev.teammates,
                newContributor: ''
            };
        });
    }

    removeContributor(idx: number) {
        this.setState((prev, props) => {
            prev.teammates.splice(idx, 1);
            return {
                teammates: prev.teammates,
            };
        });
    }

    render() {
        return (
            <div className="project-view-container">
                <div className="top-bar">
                    <div className="inputs">
                        <InputGroup
                            id="title"
                            label="Title"
                            value={this.state.title}
                            onChange={ev => this.setState({ title: ev.currentTarget.value })}
                        />
                        <InputGroup
                            id="sub-title"
                            label="Subtitle"
                            value={this.state.subtitle}
                            onChange={ev => this.setState({ subtitle: ev.currentTarget.value })}
                        />
                        <InputGroup
                                id="new-contributor"
                                label="New Contributor"
                                value={this.state.newContributor}
                                onChange={ev => this.setState({newContributor: ev.currentTarget.value})}
                            />
                        <button 
                            className="save"
                            onClick={ev => this.addContributor()}
                        >+</button>
                        <button
                            onClick={ev => this.setState((prev, props) => { 
                                return {
                                    bwTitleImage: !prev.bwTitleImage
                                };
                            })}
                        >{`Title Image ${this.state.bwTitleImage ? 'B&W' : 'Color'}`}</button>
                        <button
                            className="remove cancel"
                            onClick={ev => this.props.deleteProject()}
                        >Delete Project</button>
                    </div>
                    <div className="contributors">
                    <span>Contributors</span>
                    <div className="contributors-list">
                        {
                            this.state.teammates.length > 0 ?
                            this.state.teammates.map((t, i) => {
                                return (
                                    <div 
                                        className="contributor-group"
                                        key={`contributor-${i}`}
                                    >
                                        <span 
                                            className="contributor" 
                                            >{t}</span>
                                        <button
                                            onClick={ev => this.removeContributor(i)}
                                            title="Click to remove"
                                            className="cancel remove"
                                        >X</button>
                                    </div>
                                )
                            })
                            : <div className="contributor-group"><span className="contributor">None</span></div>
                        }
                    </div>
                    </div>
                </div>
                <div className="editors">
                    <div className="content-editor">
                        <span>Content</span>
                        <textarea
                            id="description"
                            defaultValue={this.state.description}
                            onChange={ev => this.setState({ description: ev.currentTarget.value })}></textarea>
                        <div className="button-group">
                            <button className="cancel"
                                onClick={ev => { this.props.cancelHandler() }}
                            >Cancel</button>
                            <button className="save"
                                onClick={ev => { this.pageSaved() }}
                            >Save</button>
                        </div>
                    </div>
                    <div className="image-editor">
                        <div className="image-editor-title">
                            <span>Images</span>
                        </div>
                        <ListBox
                            options={this.state.images.map(i => StringHandler.fileName(i.path))}
                            selected={this.state.selectedImage ? this.state.selectedImage.position : null}
                            onChange={i => this.setState({selectedImage: this.state.images[i]})}
                        />
                        <div className="button-group">
                            <button
                                className="remove cancel"
                                onClick={ev => this.removeImages()}
                            >Remove</button>
                            <button
                                className="move"
                                onClick={ev => this.moveImage(true)}
                            >↥</button>
                            <button
                                className="move"
                                onClick={ev => this.moveImage(false)}
                            >↧</button>
                            <button
                                className="add-new save"
                                onClick={ev => this.props.addImageHandler()}
                            >Add</button>
                        </div>
                    </div>
                </div>
            </div>
        )
    }
}
